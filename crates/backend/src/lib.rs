//! Web backend for LBR.

pub mod authentication;
pub mod domain;
pub mod error;
pub mod handlers;
pub mod queries;
pub mod schema;
pub mod schema_ichiran;
pub mod utils;

use crate::handlers::{decks, sentences, sources, words};
use authentication::{Expiration, SessionCache};
use axum::{
    extract::FromRef,
    response::{Html, IntoResponse},
    routing::{delete, get, post},
    Router,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use error::EyreResult;
use eyre::WrapErr;
use handlers::{auth, segment};
use ichiran::IchiranCli;
use lbr_web::Root;
use leptos::LeptosOptions;
use leptos_axum::LeptosRoutes;
use moka::future::Cache;
use std::{collections::HashMap, fmt::Debug, ops::Deref, path::PathBuf, sync::Arc, time::Duration};
use tokio::io::AsyncReadExt;
use tower_cookies::{CookieManagerLayer, Key};

pub type LbrPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct LbrState(Arc<LbrStateCore>);

impl Deref for LbrState {
    type Target = LbrStateCore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for LbrState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lbr")
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

impl FromRef<LbrState> for LeptosOptions {
    fn from_ref(input: &LbrState) -> Self {
        input.leptos_options.clone()
    }
}

pub async fn router(state: LbrState) -> Router<()> {
    Router::new()
        .route("/favicon.ico", get(favicon))
        .route("/license.html", get(license))
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
                                .route("/details", get(sources::get_details))
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
                                .route("/generate/:filename", get(decks::generate)),
                        ),
                )
                .nest(
                    "/sentences",
                    Router::new().nest(
                        "/:id",
                        Router::new()
                            .route(
                                "/",
                                get(sentences::get_one)
                                    .post(sentences::update)
                                    .delete(sentences::delete),
                            )
                            .route("/segment", post(sentences::segment)),
                    ),
                )
                .nest(
                    "/words",
                    Router::new().nest(
                        "/ignored",
                        Router::new()
                            .route("/", get(words::ignored_words))
                            .route("/:id", delete(words::delete_ignored_word)),
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
        .with_state(state)
}

pub async fn router_from_vars(
    lbr_database_url: &str,
    ichiran_database_url: &str,
    ichiran_cli_path: PathBuf,
    private_cookie_password: &str,
) -> eyre::Result<Router<()>> {
    // conservative pool config aimed at not using the database too much
    let lbr_pool = Pool::builder()
        .min_idle(Some(0))
        .idle_timeout(Some(Duration::from_secs(30)))
        .build(ConnectionManager::new(lbr_database_url))
        .wrap_err_with(|| format!("Failed to connect to the LBR database at {lbr_database_url}"))?;
    let ichiran_pool =
        Pool::new(ConnectionManager::new(ichiran_database_url)).wrap_err_with(|| {
            format!("Failed to connect to the ichiran database at {ichiran_database_url}")
        })?;
    let ichiran_cli = IchiranCli::new(ichiran_cli_path);
    let kanji_to_readings = match tokio::fs::File::open("./data/kanji_to_readings.bitcode").await {
        Ok(mut file) => {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).await?;
            bitcode::decode(&buf)?
        }
        Err(_) => {
            let lbr_pool = lbr_pool.clone();
            let kanji_to_readings = tokio::task::spawn_blocking(move || {
                let mut conn = lbr_pool.get()?;
                let ktr = domain::japanese::kanji_to_readings(&mut conn)?;
                EyreResult::Ok(ktr)
            })
            .await
            .wrap_err("Failed to generate kanji to readings mapping")??;
            let kanji_to_readings_bitcode = bitcode::encode(&kanji_to_readings)?;
            tokio::fs::create_dir_all("./data").await?;
            tokio::fs::write(
                "./data/kanji_to_readings.bitcode",
                kanji_to_readings_bitcode,
            )
            .await?;
            kanji_to_readings
        }
    };
    let ichiran_seq_to_word_id =
        match tokio::fs::File::open("./data/ichiran_seq_to_word_id.bitcode").await {
            Ok(mut file) => {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).await?;
                bitcode::decode(&buf)?
            }
            Err(_) => {
                let lbr_pool = lbr_pool.clone();
                let ichiran_pool = ichiran_pool.clone();
                let ichiran_seq_to_word_id = tokio::task::spawn_blocking(move || {
                    let mut lbr_conn = lbr_pool.get()?;
                    let mut ichiran_conn = ichiran_pool.get()?;
                    let istw = domain::ichiran::get_ichiran_seq_to_word_id(
                        &mut lbr_conn,
                        &mut ichiran_conn,
                    )?;
                    EyreResult::Ok(istw)
                })
                .await??;
                let ichiran_seq_to_word_id_bitcode = bitcode::encode(&ichiran_seq_to_word_id)?;
                tokio::fs::create_dir_all("./data").await?;
                tokio::fs::write(
                    "./data/ichiran_seq_to_word_id.bitcode",
                    ichiran_seq_to_word_id_bitcode,
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

    let state = LbrState(Arc::new(LbrStateCore {
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

pub async fn favicon() -> impl IntoResponse {
    include_bytes!("../../../data/favicon.ico")
}

pub async fn license() -> impl IntoResponse {
    Html(include_str!("../../../data/license.html"))
}
