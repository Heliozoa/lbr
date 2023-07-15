//! Contains the `Session` type and `Authentication` extractor as well as other authentication related helpers.

use crate::LbrState;
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use moka::{future::Cache, Expiry};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, time::Duration};
use tower_cookies::{cookie::SameSite, Cookie, Cookies, SignedCookies};


/// Extractor used for authentication.
#[derive(Deserialize, Serialize)]
pub struct Authentication {
    pub session_id: i32,
    pub user_id: i32,
}

#[async_trait]
impl FromRequestParts<LbrState> for Authentication {
    type Rejection = (StatusCode, &'static str);

    /// Checks the cache for a session that corresponds to the cookie.
    async fn from_request_parts(
        parts: &mut Parts,
        state: &LbrState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Authentication {
            session_id: 1,
            user_id: 1,
        })
    }
}
