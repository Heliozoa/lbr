pub mod client;
pub mod session;

use self::{client::Client, session::Session};
use crate::context::client::ClientBuilder;
use leptos::*;

pub fn initialise_context(cx: Scope, backend_addr: &'static str) {
    tracing::trace!("Initialising context");

    let client_builder = ClientBuilder::new(backend_addr);
    leptos::provide_context(cx, client_builder);

    let session = Session::new(cx);
    session.refresh();
    leptos::provide_context(cx, session);
}

pub fn get_client(cx: Scope) -> Client {
    leptos::expect_context::<ClientBuilder>(cx).build(cx)
}

pub fn get_session(cx: Scope) -> Session {
    leptos::expect_context::<Session>(cx)
}

pub fn refresh_session(cx: Scope) {
    leptos::use_context::<Session>(cx).map(|s| s.refresh());
}
