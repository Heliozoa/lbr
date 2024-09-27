//! LBR server error type.

use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
};
use lbr_api::response as res;

pub type LbrResult<T> = Result<T, LbrError>;

pub struct LbrError(eyre::Error);

impl<E> From<E> for LbrError
where
    E: Into<eyre::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl IntoResponse for LbrError {
    fn into_response(self) -> axum::response::Response {
        let err = res::Error {
            message: format!("{:#?}", self.0),
        };
        let body = serde_json::to_string(&err).expect("failed to serialize response");
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .expect("failed to construct response")
            .into_response()
    }
}

pub type EyreResult<T> = Result<T, eyre::Report>;
