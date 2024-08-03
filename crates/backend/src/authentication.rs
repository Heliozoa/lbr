//! Contains the `Session` type and `Authentication` extractor as well as other authentication related helpers.

use crate::LbrState;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    RequestPartsExt,
};
use moka::{future::Cache, Expiry};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt::Debug, time::Duration};
use tower_cookies::{cookie::SameSite, Cookie, Cookies, SignedCookies};

pub type SessionCache = Cache<i32, Session>;

/// Session stored in the server's cache.
#[derive(Clone)]
pub struct Session {
    /// The user's database id.
    user_id: i32,
}

/// Basic cache expiration policy that refreshes after reads and writes.
pub struct Expiration {
    pub days: Duration,
}

impl Expiration {
    pub fn new(days: u32) -> Self {
        Self {
            days: chrono::Duration::days(days.into())
                .to_std()
                .expect("Invalid duration"),
        }
    }
}

impl<K, V> Expiry<K, V> for Expiration {
    fn expire_after_create(
        &self,
        _key: &K,
        _value: &V,
        _current_time: std::time::Instant,
    ) -> Option<std::time::Duration> {
        Some(self.days)
    }

    fn expire_after_read(
        &self,
        _key: &K,
        _value: &V,
        _current_time: std::time::Instant,
        _current_duration: Option<std::time::Duration>,
        _last_modified_at: std::time::Instant,
    ) -> Option<Duration> {
        Some(self.days)
    }

    fn expire_after_update(
        &self,
        _key: &K,
        _value: &V,
        _current_time: std::time::Instant,
        _current_duration: Option<std::time::Duration>,
    ) -> Option<std::time::Duration> {
        Some(self.days)
    }
}

/// The cookie that is stored signed on the user's browser for authentication.
#[derive(Deserialize, Serialize)]
struct SessionCookie {
    user_id: i32,
    /// Session's id in the cache
    session_id: i32,
}

impl SessionCookie {
    /// The name of the cookie in the browser.
    const NAME: &'static str = lbr_api::SESSION_COOKIE_NAME;

    /// Creates a new session cookie with a random session id.
    fn new(user_id: i32) -> Self {
        Self {
            user_id,
            session_id: rand::random(),
        }
    }

    /// Tries to extract the cookie from private cookies.
    /// Removes cookies that exist but fail to parse.
    fn from_signed_cookies(signed_cookies: &SignedCookies<'_>) -> Option<Self> {
        let cookie = signed_cookies.get(SessionCookie::NAME)?;
        match serde_json::from_str::<SessionCookie>(cookie.value()) {
            Ok(session_cookie) => Some(session_cookie),
            Err(_err) => {
                // found cookie but it was malformed for whatever reason, remove it
                remove_session_cookie(signed_cookies);
                None
            }
        }
    }
}

/// Extractor used for authentication.
#[derive(Deserialize, Serialize)]
pub struct Authentication {
    pub session_id: i32,
    pub user_id: i32,
}

impl Debug for Authentication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_id)
    }
}

#[async_trait]
impl FromRequestParts<LbrState> for Authentication {
    type Rejection = (StatusCode, &'static str);

    /// Checks the cache for a session that corresponds to the cookie.
    async fn from_request_parts(
        parts: &mut Parts,
        state: &LbrState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = parts.extract::<Cookies>().await?;
        let signed_cookies = cookies.signed(&state.private_cookie_key);
        let session_cookie = SessionCookie::from_signed_cookies(&signed_cookies)
            .ok_or((StatusCode::UNAUTHORIZED, "Not logged in"))?;
        match state.sessions.get(&session_cookie.session_id).await {
            Some(session) => Ok(Authentication {
                session_id: session_cookie.session_id,
                user_id: session.user_id,
            }),
            None => {
                // has cookie but no session
                // todo: reject
                remove_session_cookie(&signed_cookies);
                let session_id =
                    save_session(session_cookie.user_id, signed_cookies, &state.sessions)
                        .await
                        .map_err(|_| (StatusCode::UNAUTHORIZED, "Failed to save session"))?;
                Ok(Authentication {
                    session_id,
                    user_id: session_cookie.user_id,
                })
            }
        }
    }
}

fn build_cookie(value: impl Into<Cow<'static, str>>) -> Cookie<'static> {
    Cookie::build((SessionCookie::NAME, value))
        .path("/")
        .secure(false)
        .http_only(true)
        .same_site(SameSite::Strict)
        .build()
}

/// Saves a new session for the user to both the cookies and server cache.
pub async fn save_session(
    user_id: i32,
    signed_cookies: SignedCookies<'_>,
    sessions: &SessionCache,
) -> eyre::Result<i32> {
    let session_cookie = SessionCookie::new(user_id);
    let cookie_value = serde_json::to_string(&session_cookie)?;
    let cookie = build_cookie(cookie_value);
    signed_cookies.add(cookie);
    sessions
        .insert(session_cookie.session_id, Session { user_id })
        .await;
    Ok(session_cookie.session_id)
}

fn remove_session_cookie(signed_cookies: &SignedCookies<'_>) {
    let cookie = build_cookie("");
    signed_cookies.remove(cookie);
}

/// Forgets the session.
pub async fn forget_session(
    session_id: i32,
    signed_cookies: &SignedCookies<'_>,
    sessions: &SessionCache,
) -> eyre::Result<()> {
    remove_session_cookie(signed_cookies);
    sessions.remove(&session_id).await;
    Ok(())
}

pub fn hash_password(password: &str) -> eyre::Result<String> {
    let argon = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| eyre::eyre!("Failed to hash password"))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> eyre::Result<()> {
    let argon = Argon2::default();
    let password_hash = PasswordHash::new(password_hash)
        .map_err(|_| eyre::eyre!("Failed to create password hash"))?;
    argon
        .verify_password(password.as_bytes(), &password_hash)
        .map_err(|_| eyre::eyre!("Failed to verify password"))?;
    Ok(())
}
