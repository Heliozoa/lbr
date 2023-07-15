pub mod client;
pub mod session;

use self::{client::Client, session::Session};
use crate::context::client::ClientBuilder;
use leptos::*;

pub fn initialise_context(cx: Scope) {
    tracing::trace!("Initialising context");

    let client_builder = ClientBuilder::new();
    leptos::provide_context(cx, client_builder);

    let session = Session::new(cx);
    leptos::provide_context(cx, session);
}

pub fn get_client(cx: Scope) -> Client {
    if cfg!(feature = "ssr") {
        ClientBuilder::new().build(cx)
    } else {
        leptos::expect_context::<ClientBuilder>(cx).build(cx)
    }
}

pub fn get_session(cx: Scope) -> Session {
    if cfg!(feature = "ssr") {
        Session::new(cx)
    } else {
        leptos::expect_context::<Session>(cx)
    }
}
