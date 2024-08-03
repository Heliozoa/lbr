//! Session context for authentication.

use super::get_client;
use leptos::prelude::*;
use send_wrapper::SendWrapper;

#[derive(Clone, Copy)]
pub struct Session {
    pub user_id: Action<(), Option<i32>>,
}

impl Session {
    pub fn new() -> Self {
        let user_id = Action::new(move |()| async move {
            if !cfg!(feature = "ssr") {
                let client = get_client();
                match SendWrapper::new(client.current_user()).await {
                    Ok(Some(user)) => {
                        tracing::info!("Current user: {user:#?}");
                        Some(user)
                    }
                    _ => {
                        tracing::info!("Current user: none");
                        None
                    }
                }
            } else {
                None
            }
        });
        if !cfg!(feature = "ssr") {
            user_id.dispatch(());
        }
        Self { user_id }
    }

    pub fn logged_in(&self) -> Option<bool> {
        let user_id = &*self.user_id.value().read();
        match user_id {
            Some(Some(_user_id)) => Some(true),
            Some(None) => Some(false),
            None => None,
        }
    }

    pub fn refresh(&self) {
        self.user_id.dispatch(());
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
