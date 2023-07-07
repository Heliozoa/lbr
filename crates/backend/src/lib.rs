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
    extract::State,
    http::{header, HeaderValue, Method},
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
use moka::future::Cache;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tower_cookies::{CookieManagerLayer, Key};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub type LbrState = State<Arc<LbrStateInner>>;
pub type LbrPool = Pool<ConnectionManager<PgConnection>>;

pub struct LbrStateInner {
    pub lbr_pool: LbrPool,
    pub ichiran_pool: LbrPool,
    pub ichiran_cli: IchiranCli,
    pub kanji_to_readings: HashMap<String, Vec<String>>,
    pub ichiran_seq_to_word_id: HashMap<i32, i32>,
    pub private_cookie_key: Key,
    pub sessions: SessionCache,
}

pub async fn router(state: Arc<LbrStateInner>) -> Router {
    let router = Router::new().nest(
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
            .with_state(state)
            .layer(CookieManagerLayer::new())
            .layer(
                CorsLayer::new()
                    .allow_methods(vec![Method::GET, Method::POST, Method::DELETE])
                    .allow_headers(vec![header::CONTENT_TYPE])
                    .allow_origin(AllowOrigin::exact(HeaderValue::from_static(
                        "http://localhost:8080",
                    )))
                    .allow_credentials(true),
            ),
    );
    router
}

pub async fn router_from_vars(
    lbr_database_url: &str,
    ichiran_database_url: &str,
    ichiran_cli_path: PathBuf,
    private_cookie_password: &str,
) -> eyre::Result<Router> {
    let lbr_pool = Pool::new(ConnectionManager::new(lbr_database_url))
        .wrap_err_with(|| format!("Failed to connect to the LBR database at {lbr_database_url}"))?;
    let ichiran_pool =
        Pool::new(ConnectionManager::new(ichiran_database_url)).wrap_err_with(|| {
            format!("Failed to connect to the ichiran database at {ichiran_database_url}")
        })?;
    let ichiran_cli = IchiranCli::new(ichiran_cli_path);
    let kanji_to_readings = domain::japanese::kanji_to_readings(lbr_pool.clone())
        .await
        .wrap_err("Failed to generate kanji to readings mapping")?;
    let ichiran_seq_to_word_id =
        domain::ichiran::get_ichiran_seq_to_word_id(lbr_pool.clone(), ichiran_pool.clone()).await?;
    let private_cookie_key = Key::from(private_cookie_password.as_bytes());
    let sessions = Cache::builder()
        .max_capacity(100_000_000)
        .expire_after(Expiration::new(4))
        .build();

    let state = Arc::new(LbrStateInner {
        lbr_pool,
        ichiran_pool,
        ichiran_cli,
        kanji_to_readings,
        ichiran_seq_to_word_id,
        private_cookie_key,
        sessions,
    });
    let router = self::router(state).await;
    Ok(router)
}
