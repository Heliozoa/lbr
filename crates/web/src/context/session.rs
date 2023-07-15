//! Session context for authentication.

use super::get_client;
use leptos::*;

#[derive(Clone, Copy)]
pub struct Session {
    pub user_id: Action<(), Option<i32>>,
}

impl Session {
    pub fn new(cx: Scope) -> Self {
        let user_id = leptos::create_action(cx, move |()| async move {
            let client = get_client(cx);
            match client.current_user().await {
                Ok(Some(user)) => {
                    tracing::info!("Current user: {user:#?}");
                    Some(user)
                }
                _ => {
                    tracing::info!("Current user: none");
                    None
                }
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
