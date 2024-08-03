pub mod client;
pub mod session;

use self::{client::Client, session::Session};
use leptos::prelude::*;

pub fn initialise_context() {
    tracing::trace!("initialising context");

    leptos_meta::provide_meta_context();
    leptos::context::provide_context(Session::new());
    leptos::context::provide_context(Client::new());
}

pub fn get_client() -> Client {
    Client::new()
}

pub fn get_session() -> Session {
    if cfg!(feature = "ssr") {
        // returning a "dummy" session within the server
        Session::new()
    } else {
        let owner = Owner::current().unwrap();
        owner.with(move || leptos::prelude::expect_context::<Session>())
    }
}

pub fn get_session_cx(owner: Owner) -> Session {
    if cfg!(feature = "ssr") {
        // returning a "dummy" session within the server
        Session::new()
    } else {
        owner.with(move || leptos::prelude::expect_context::<Session>())
    }
}

pub fn refresh_session() {
    get_session().refresh();
}
