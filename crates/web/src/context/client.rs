//! Client context for communicating with the backend.

use crate::error::{WebError, WebResult};
use leptos::*;
use reqwasm::http::Response;
use web_sys::RequestCredentials;

#[derive(Clone, Copy)]
pub(super) struct ClientBuilder {}

impl ClientBuilder {
    pub(super) fn new() -> Self {
        Self {}
    }

    pub(super) fn build(self, cx: Scope) -> Client {
        Client { cx }
    }
}

#[derive(Clone, Copy)]
pub struct Client {
    cx: Scope,
}

/// Non-API methods
impl Client {
    async fn assert_success(&self, res: &Response) -> eyre::Result<()> {
        match res.status() {
            100..=399 => Ok(()),
            401 => {
                tracing::warn!("Server unexpectedly returned 401");
                // not logged in according to server, so refresh logged in status
                self.refresh_session();
                Err(eyre::eyre!("Unauthorized"))
            }
            code => {
                panic!()
            }
        }
    }

    fn refresh_session(&self) {
        let session = super::get_session(self.cx);
        if !session.user_id.pending().get_untracked() {
            session.user_id.dispatch(());
        }
    }
}

/// API methods
impl Client {
    pub async fn current_user(&self) -> WebResult<Option<i32>> {
        tracing::info!("Fetching current user");

        let res = reqwasm::http::Request::get(&format!("/api/auth/current"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let current_user: Option<i32> = res.json().await.map_err(WebError::from)?;

        Ok(current_user)
    }

    pub async fn get_decks(&self) -> WebResult<Vec<String>> {
        tracing::info!("Fetching decks");

        let res = reqwasm::http::Request::get(&format!("/api/decks"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let decks = res.json().await.map_err(WebError::from)?;

        tracing::info!("Fetched decks: {decks:?}");
        Ok(decks)
    }
}
