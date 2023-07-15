//! Web backend for LBR.

use axum::{
    async_trait,
    body::{boxed, Body, BoxBody},
    extract::{FromRequestParts, State},
    http::{request::Parts, Request, Response, StatusCode, Uri},
    response::{IntoResponse, Response as AxumResponse},
    routing::get,
    Json, Router,
};
use lbr_web::Root;
use leptos::LeptosOptions;
use leptos_axum::LeptosRoutes;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
use tower_http::services::ServeDir;

pub async fn router() -> Router<()> {
    let state = leptos::get_configuration(None)
        .await
        .unwrap()
        .leptos_options;
    let router = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/auth", Router::new().route("/current", get(current)))
                .nest("/decks", Router::new().route("/", get(get_all))),
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
        .fallback(file_and_error_handler)
        .with_state(state);
    router
}

pub async fn current(user: Option<Authentication>) -> Json<Option<i32>> {
    Json(user.map(|u| u.user_id))
}

pub async fn get_all(_: Authentication) -> Json<Vec<String>> {
    Json(vec!["a".to_string(), "b".to_string()])
}

pub async fn file_and_error_handler(
    uri: Uri,
    State(options): State<LeptosOptions>,
) -> AxumResponse {
    let root = options.site_root.clone();
    let res = get_static_file(uri.clone(), &root).await.unwrap();
    res.into_response()
}

async fn get_static_file(uri: Uri, root: &str) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder()
        .uri(uri.clone())
        .body(Body::empty())
        .unwrap();
    match ServeDir::new(root).oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {err}"),
        )),
    }
}

#[derive(Deserialize, Serialize)]
pub struct Authentication {
    pub session_id: i32,
    pub user_id: i32,
}

#[async_trait]
impl FromRequestParts<LeptosOptions> for Authentication {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(_: &mut Parts, _: &LeptosOptions) -> Result<Self, Self::Rejection> {
        Ok(Authentication {
            session_id: 1,
            user_id: 1,
        })
    }
}
