//! Session context for authentication.

use super::get_client;
use leptos::*;

#[derive(Clone, Copy)]
pub struct Session {
    pub user_id: Action<(), Option<i32>>,
}

impl Session {
    pub fn new() -> Self {
        let user_id = leptos::create_action(move |()| async move {
            let client = get_client();
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
            tracing::info!("pending");
            None
        } else {
            let val = self.user_id.value().get();
            tracing::info!("val {val:?}");
            match val {
                Some(Some(_user_id)) => Some(true),
                Some(None) => Some(false),
                None => None,
            }
        }
    }
}
