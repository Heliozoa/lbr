//! Web backend for LBR.

pub mod authentication;
pub mod handlers;

use crate::handlers::{auth, decks};
use axum::{extract::FromRef, routing::get, Router};
use lbr_web::Root;
use leptos::LeptosOptions;
use leptos_axum::LeptosRoutes;
use std::{ops::Deref, sync::Arc};
use tower_cookies::Key;

#[derive(Clone)]
pub struct LbrState(Arc<LbrStateCore>);

impl Deref for LbrState {
    type Target = LbrStateCore;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct LbrStateCore {
    pub private_cookie_key: Key,
    pub leptos_options: LeptosOptions,
}

impl FromRef<LbrState> for LeptosOptions {
    fn from_ref(input: &LbrState) -> Self {
        input.leptos_options.clone()
    }
}

pub async fn router(state: LbrState) -> Router<()> {
    let router = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/auth", Router::new().route("/current", get(auth::current)))
                .nest("/decks", Router::new().route("/", get(decks::get_all))),
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

pub async fn router_from_vars(private_cookie_password: &str) -> eyre::Result<Router<()>> {
    let private_cookie_key = Key::from(private_cookie_password.as_bytes());
    let leptos_options = leptos::get_configuration(None)
        .await
        .unwrap()
        .leptos_options;

    let state = LbrState(Arc::new(LbrStateCore {
        private_cookie_key,
        leptos_options,
    }));
    let router = self::router(state).await;
    Ok(router)
}
