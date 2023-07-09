//! Web backend for LBR.

pub mod authentication;
pub mod domain;
pub mod error;
pub mod handlers;
pub mod schema;
pub mod schema_ichiran;
pub mod utils;

use crate::handlers::{decks, sentences, sources};
use authentication::{Expiration, SessionCache};
use axum::{
    extract::{FromRef, State},
    routing::{get, post},
    Router,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use eyre::WrapErr;
use handlers::{auth, segment};
use ichiran::IchiranCli;
use lbr_web::Root;
use leptos::LeptosOptions;
use leptos_axum::LeptosRoutes;
use moka::future::Cache;
use std::{collections::HashMap, ops::Deref, path::PathBuf, sync::Arc};
use tokio::io::AsyncReadExt;
use tower_cookies::{CookieManagerLayer, Key};

pub type LbrState = State<LbrStateInner>;
pub type LbrPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct LbrStateInner(Arc<LbrStateCore>);

impl Deref for LbrStateInner {
    type Target = LbrStateCore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct LbrStateCore {
    pub lbr_pool: LbrPool,
    pub ichiran_pool: LbrPool,
    pub ichiran_cli: IchiranCli,
    pub kanji_to_readings: HashMap<String, Vec<String>>,
    pub ichiran_seq_to_word_id: HashMap<i32, i32>,
    pub private_cookie_key: Key,
    pub sessions: SessionCache,
    pub leptos_options: LeptosOptions,
}

impl FromRef<LbrStateInner> for LeptosOptions {
    fn from_ref(input: &LbrStateInner) -> Self {
        input.leptos_options.clone()
    }
}

pub async fn router(state: LbrStateInner) -> Router<()> {
    let router = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest(
                    "/auth",
                    Router::new()
                        .route("/register", post(auth::register))
                        .route("/login", post(auth::login))
                        .route("/current", get(auth::current))
                        .route("/logout", post(auth::logout)),
                )
                .nest(
                    "/sources",
                    Router::new()
                        .route("/", get(sources::get_all).post(sources::insert))
                        .nest(
                            "/:id",
                            Router::new()
                                .route(
                                    "/",
                                    get(sources::get_one)
                                        .post(sources::update)
                                        .delete(sources::delete),
                                )
                                .route("/sentence", post(sources::add_sentence)),
                        ),
                )
                .nest(
                    "/decks",
                    Router::new()
                        .route("/", get(decks::get_all).post(decks::insert))
                        .nest(
                            "/:id",
                            Router::new()
                                .route(
                                    "/",
                                    get(decks::get_one)
                                        .post(decks::update)
                                        .delete(decks::delete),
                                )
                                .route("/sources", post(decks::update_sources))
                                .route("/generate/:filename", get(decks::generate)),
                        ),
                )
                .nest(
                    "/sentences",
                    Router::new()
                        .route("/", get(sentences::get).post(sentences::insert))
                        .route(
                            "/:id",
                            get(sentences::get_one)
                                .post(sentences::update)
                                .delete(sentences::delete),
                        ),
                )
                .route("/segment", post(segment::segment))
                .layer(CookieManagerLayer::new()),
        )
        .leptos_routes(
            &state,
            leptos_axum::generate_route_list(|cx| {
                leptos::view! { cx, <Root/> }
            })
            .await,
            |cx| {
                leptos::view! { cx, <Root/> }
            },
        )
        .fallback(handlers::file_and_error_handler)
        .with_state(state);
    router
}

pub async fn router_from_vars(
    lbr_database_url: &str,
    ichiran_database_url: &str,
    ichiran_cli_path: PathBuf,
    private_cookie_password: &str,
) -> eyre::Result<Router<()>> {
    let lbr_pool = Pool::new(ConnectionManager::new(lbr_database_url))
        .wrap_err_with(|| format!("Failed to connect to the LBR database at {lbr_database_url}"))?;
    let ichiran_pool =
        Pool::new(ConnectionManager::new(ichiran_database_url)).wrap_err_with(|| {
            format!("Failed to connect to the ichiran database at {ichiran_database_url}")
        })?;
    let ichiran_cli = IchiranCli::new(ichiran_cli_path);
    let kanji_to_readings = match tokio::fs::File::open("./data/kanji_to_readings.json").await {
        Ok(mut file) => {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;
            let kanji_to_readings = serde_json::from_slice(&buf)?;
            kanji_to_readings
        }
        Err(_) => {
            let kanji_to_readings = domain::japanese::kanji_to_readings(lbr_pool.clone())
                .await
                .wrap_err("Failed to generate kanji to readings mapping")?;
            let kanji_to_readings_json = serde_json::to_string_pretty(&kanji_to_readings)?;
            tokio::fs::create_dir_all("./data").await?;
            tokio::fs::write("./data/kanji_to_readings.json", kanji_to_readings_json).await?;
            kanji_to_readings
        }
    };
    let ichiran_seq_to_word_id = match tokio::fs::File::open("./data/ichiran_seq_to_word_id.json")
        .await
    {
        Ok(mut file) => {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;
            let ichiran_seq_to_word_id = serde_json::from_slice(&buf)?;
            ichiran_seq_to_word_id
        }
        Err(_) => {
            let ichiran_seq_to_word_id =
                domain::ichiran::get_ichiran_seq_to_word_id(lbr_pool.clone(), ichiran_pool.clone())
                    .await?;
            let ichiran_seq_to_word_id_json =
                serde_json::to_string_pretty(&ichiran_seq_to_word_id)?;
            tokio::fs::create_dir_all("./data").await?;
            tokio::fs::write(
                "./data/ichiran_seq_to_word_id.json",
                ichiran_seq_to_word_id_json,
            )
            .await?;
            ichiran_seq_to_word_id
        }
    };
    let private_cookie_key = Key::from(private_cookie_password.as_bytes());
    let sessions = Cache::builder()
        .max_capacity(100_000_000)
        .expire_after(Expiration::new(4))
        .build();
    let leptos_options = leptos::get_configuration(None)
        .await
        .unwrap()
        .leptos_options;

    let state = LbrStateInner(Arc::new(LbrStateCore {
        lbr_pool,
        ichiran_pool,
        ichiran_cli,
        kanji_to_readings,
        ichiran_seq_to_word_id,
        private_cookie_key,
        sessions,
        leptos_options,
    }));
    let router = self::router(state).await;
    Ok(router)
}
