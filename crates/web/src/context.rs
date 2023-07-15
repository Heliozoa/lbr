use crate::error::{WebError, WebResult};
use leptos::*;
use web_sys::RequestCredentials;

pub fn initialise_context(cx: Scope) {
    tracing::trace!("Initialising context");

    let session = Session::new(cx);
    leptos::provide_context(cx, session);
}

pub fn get_session(cx: Scope) -> Session {
    if cfg!(feature = "ssr") {
        Session::new(cx)
    } else {
        leptos::expect_context::<Session>(cx)
    }
}

pub struct Client;

impl Client {
    pub async fn current_user(&self) -> WebResult<Option<i32>> {
        let res = reqwasm::http::Request::get(&format!("/api/auth/current"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        let current_user: Option<i32> = res.json().await.map_err(WebError::from)?;
        Ok(current_user)
    }

    pub async fn get_decks(&self) -> WebResult<Vec<String>> {
        let res = reqwasm::http::Request::get(&format!("/api/decks"))
            .credentials(RequestCredentials::Include)
            .send()
            .await
            .map_err(WebError::from)?;
        let decks = res.json().await.map_err(WebError::from)?;
        Ok(decks)
    }
}

#[derive(Clone, Copy)]
pub struct Session {
    pub user_id: Action<(), Option<i32>>,
}

impl Session {
    pub fn new(cx: Scope) -> Self {
        let user_id = leptos::create_action(cx, move |()| async move {
            match Client.current_user().await {
                Ok(user) => user,
                _ => None,
            }
        });
        if !cfg!(feature = "ssr") {
            user_id.dispatch(());
        }
        Self { user_id }
    }

    pub fn logged_in(&self) -> Option<bool> {
        if self.user_id.pending().get() {
            None
        } else {
            match self.user_id.value().get() {
                Some(Some(_user_id)) => Some(true),
                Some(None) => Some(false),
                None => None,
            }
        }
    }
}
