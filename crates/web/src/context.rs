pub mod client;
pub mod session;

use self::{client::Client, session::Session};
use leptos::*;

pub fn initialise_context() {
    tracing::trace!("Initialising context");

    leptos::provide_context(Session::new());
}

pub fn get_client() -> Client {
    Client
}

pub fn get_session() -> Session {
    if cfg!(feature = "ssr") {
        Session::new()
    } else {
        let cx = Owner::current().unwrap();
        leptos::with_owner(cx, move || leptos::expect_context::<Session>())
    }
}

pub fn get_session_cx(cx: Owner) -> Session {
    if cfg!(feature = "ssr") {
        Session::new()
    } else {
        leptos::with_owner(cx, move || leptos::expect_context::<Session>())
    }
}
