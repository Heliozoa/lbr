//! Client context for communicating with the backend.

use crate::error::{WebError, WebResult};
use lbr_api::{request as req, response as res};
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
                let bytes = res.binary().await.unwrap_or_default();
                let body = match serde_json::from_slice::<res::Error>(&bytes) {
                    Ok(error) => error.message.into(),
                    Err(_) => String::from_utf8_lossy(bytes.as_slice()),
                };
                Err(eyre::eyre!("Request failed: HTTP {code} {body}"))
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
    pub async fn register(&self, email: &str, password: &str) -> WebResult<()> {
        tracing::info!("Registering {email}");

        let register = req::Register {
            email: email.into(),
            password: password.into(),
        };
        let json = serde_json::to_string(&register).map_err(WebError::from)?;
        let res = reqwasm::http::Request::post(&format!("/api/auth/register"))
            .credentials(RequestCredentials::Include)
            .body(json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        tracing::info!("Registered {email}");
        Ok(())
    }

    pub async fn login(&self, email: &str, password: &str) -> WebResult<()> {
        tracing::info!("Logging in as {email}");

        let login = req::Login {
            email: email.into(),
            password: password.into(),
        };
        let json = serde_json::to_string(&login).map_err(WebError::from)?;
        let res = reqwasm::http::Request::post(&format!("/api/auth/login"))
            .credentials(RequestCredentials::Include)
            .body(json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        self.refresh_session();

        tracing::info!("Logged in as {email}");
        Ok(())
    }

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

    pub async fn logout(&self) -> WebResult<()> {
        tracing::info!("Logging out");

        let res = reqwasm::http::Request::post(&format!("/api/auth/logout"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        self.refresh_session();

        tracing::info!("Logged out");
        Ok(())
    }

    pub async fn get_sources(&self) -> WebResult<Vec<res::Source>> {
        tracing::info!("Fetching sources");

        let res = reqwasm::http::Request::get(&format!("/api/sources"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let sources = res.json().await.map_err(WebError::from)?;

        tracing::info!("Fetched sources: {sources:?}");
        Ok(sources)
    }

    pub async fn new_source(&self, name: &str) -> WebResult<i32> {
        tracing::info!("Creating source {name}");

        let json =
            serde_json::to_string(&req::NewSource { name: name.into() }).map_err(WebError::from)?;
        let res = reqwasm::http::Request::post(&format!("/api/sources"))
            .credentials(RequestCredentials::Include)
            .body(json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let id = read_i32(&res).await?;

        tracing::info!("Created source {name}");
        Ok(id)
    }

    pub async fn get_source(&self, id: i32) -> WebResult<res::Source> {
        tracing::info!("Fetching source {id}");

        let res = reqwasm::http::Request::get(&format!("/api/sources/{id}"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let source = res.json().await.map_err(WebError::from)?;

        tracing::info!("Fetched source {id}: {source:?}");
        Ok(source)
    }

    pub async fn get_source_details(&self, id: i32) -> WebResult<res::SourceDetails> {
        tracing::info!("Fetching source {id}");

        let res = reqwasm::http::Request::get(&format!("/api/sources/{id}/details"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let source = res.json().await.map_err(WebError::from)?;

        tracing::info!("Fetched source {id}: {source:?}");
        Ok(source)
    }

    pub async fn update_source(&self, id: i32, name: &str) -> WebResult<()> {
        tracing::info!("Updating source {id}");

        let json = serde_json::to_string(&req::UpdateSource { name: name.into() })
            .map_err(WebError::from)?;
        let res = reqwasm::http::Request::post(&format!("/api/sources/{id}"))
            .credentials(RequestCredentials::Include)
            .body(json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        Ok(())
    }

    pub async fn delete_source(&self, id: i32) -> WebResult<()> {
        tracing::info!("Deleting source {id}");

        let res = reqwasm::http::Request::delete(&format!("/api/sources/{id}"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        tracing::info!("Deleted source {id}");
        Ok(())
    }

    pub async fn get_sentence(&self, id: i32) -> WebResult<res::SentenceDetails> {
        tracing::info!("Fetching sentence {id}");

        let res = reqwasm::http::Request::get(&format!("/api/sentences/{id}"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let sentence = res.json().await.map_err(WebError::from)?;

        tracing::info!("Fetched sentence {id}");
        Ok(sentence)
    }

    pub async fn delete_sentence(&self, id: i32) -> WebResult<()> {
        tracing::info!("Deleting sentence {id}");

        let res = reqwasm::http::Request::delete(&format!("/api/sentences/{id}"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        tracing::info!("Deleted sentence {id}");
        Ok(())
    }

    pub async fn get_decks(&self) -> WebResult<Vec<res::Deck>> {
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

    pub async fn new_deck(&self, name: &str) -> WebResult<i32> {
        tracing::info!("Creating deck {name}");

        let json =
            serde_json::to_string(&req::NewDeck { name: name.into() }).map_err(WebError::from)?;
        let res = reqwasm::http::Request::post(&format!("/api/decks"))
            .credentials(RequestCredentials::Include)
            .body(json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let id = read_i32(&res).await?;

        tracing::info!("Created deck {name}");
        Ok(id)
    }

    pub async fn get_deck(&self, id: i32) -> WebResult<res::DeckDetails> {
        tracing::info!("Fetching decks");

        let res = reqwasm::http::Request::get(&format!("/api/decks/{id}"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let deck = res.json().await.map_err(WebError::from)?;

        tracing::info!("Fetched deck {id}: {deck:?}");
        Ok(deck)
    }

    pub async fn update_deck(&self, id: i32, name: &str, sources: &[i32]) -> WebResult<()> {
        tracing::info!("Updating sources for deck {id}");

        let json = serde_json::to_string(&req::UpdateDeck {
            name: name.into(),
            included_sources: sources.into(),
        })
        .map_err(WebError::from)?;
        let res = reqwasm::http::Request::post(&format!("/api/decks/{id}"))
            .credentials(RequestCredentials::Include)
            .body(json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        tracing::info!("Updated sources for deck {id}");
        Ok(())
    }

    pub fn generate_deck_url(&self, id: i32, filename: &str) -> String {
        format!("/api/decks/{id}/generate/{filename}")
    }

    pub async fn delete_deck(&self, id: i32) -> WebResult<()> {
        tracing::info!("Deleting deck {id}");

        let res = reqwasm::http::Request::delete(&format!("/api/decks/{id}"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        tracing::info!("Deleted deck {id}");
        Ok(())
    }

    pub async fn segment_paragraph(
        &self,
        source_id: i32,
        paragraph: &str,
    ) -> WebResult<Vec<res::SegmentedSentence>> {
        tracing::info!("Segmenting paragraph {paragraph}");

        let json = serde_json::to_string(&req::Paragraph {
            source_id,
            paragraph: paragraph.into(),
        })
        .map_err(WebError::from)?;
        let res = reqwasm::http::Request::post("/api/segment")
            .credentials(RequestCredentials::Include)
            .body(json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let segmented = res.json().await.map_err(WebError::from)?;

        tracing::info!("Segmented paragraph {paragraph}");
        Ok(segmented)
    }

    pub async fn segment_sentence(&self, sentence_id: i32) -> WebResult<res::SegmentedSentence> {
        tracing::info!("Segmenting sentence {sentence_id}");

        let res = reqwasm::http::Request::post(&format!("/api/sentences/{sentence_id}/segment"))
            .credentials(RequestCredentials::Include)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;
        let segmented = res.json().await.map_err(WebError::from)?;

        tracing::info!("Segmented sentence {sentence_id}");
        Ok(segmented)
    }

    pub async fn new_sentence(
        &self,
        source_id: i32,
        sentence: &req::SegmentedSentence,
    ) -> WebResult<()> {
        tracing::info!("Sending sentence '{}'", sentence.sentence);

        let json = serde_json::to_string(&sentence).map_err(WebError::from)?;
        let res = reqwasm::http::Request::post(&format!("/api/sources/{source_id}/sentence",))
            .credentials(RequestCredentials::Include)
            .body(json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        tracing::info!("Sent sentence {}", sentence.sentence);
        Ok(())
    }

    pub async fn update_sentence(
        &self,
        sentence_id: i32,
        sentence: &req::SegmentedSentence,
    ) -> WebResult<()> {
        tracing::info!("Sending sentence '{}'", sentence.sentence);

        let json = serde_json::to_string(&sentence).map_err(WebError::from)?;
        let res = reqwasm::http::Request::post(&format!("/api/sentences/{sentence_id}"))
            .credentials(RequestCredentials::Include)
            .body(json)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(WebError::from)?;
        self.assert_success(&res).await?;

        tracing::info!("Sent sentence {}", sentence.sentence);
        Ok(())
    }
}

async fn read_i32(res: &Response) -> WebResult<i32> {
    let text = res.text().await.map_err(WebError::from)?;
    let number = text.parse().map_err(WebError::from)?;
    Ok(number)
}
