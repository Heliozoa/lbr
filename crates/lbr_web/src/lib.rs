#![allow(clippy::unit_arg)]

pub mod components;
pub mod context;
pub mod error;
pub mod pages;
pub mod utils;

use components::*;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, ParamSegment, StaticSegment};
use pages::*;

/// Wraps the content in a basic layout and a final fallback error boundary which should never actually trigger
#[component]
pub fn App() -> impl IntoView {
    tracing::info!("Rendering app");

    context::initialise_context();
    leptos_meta::provide_meta_context();

    let fallback = move |errors: ArcRwSignal<Errors>| {
        errors
            .get_untracked()
            .into_iter()
            .map(|(_key, err)| {
                view! { <div>{format!("Unhandled error: {err}")}</div>}
            })
            .collect_view()
    };

    view! {
            <Stylesheet id="lbr" href="/pkg/lbr.css"/>
            <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
            <Meta name="description" content="LBR is an application for studying Japanese"/>
            <Title text="LBR"/>
            <div class="is-flex is-flex-direction-column" style="min-height: 100vh">
                <div class="section is-flex is-flex-grow-1">
                    <div class="container">
                        <ErrorBoundary fallback>
                            <Content/>
                        </ErrorBoundary>
                    </div>
                </div>
                <footer class="footer">
                    <div class="container">
                        <a href="https://github.com/Heliozoa/lbr">"Source code"</a>
                        " / "
                        <a rel="external" href="license.html">"Third party license information"</a>
                    </div>
                </footer>
            </div>
    }
}

/// Contains the navbar and router
#[component]
pub fn Content() -> impl IntoView {
    view! {
        <Router>
            <Navbar/>
            <main>
                <h1 class="title">"LBR"</h1>
                <FlatRoutes fallback=|| "Page not found.">
                    <Route
                        path=StaticSegment("/")
                        view=Home
                    />
                    <Route
                        path=(StaticSegment("source"), StaticSegment("new"))
                        view=SourceNew
                    />
                    <Route
                        path=(StaticSegment("source"), ParamSegment("source_id"))
                        view=Source
                    />
                    <Route
                        path=(StaticSegment("source"), ParamSegment("source_id"), StaticSegment("add-sentences"))
                        view=SourceAddSentences
                    />
                    <Route
                        path=(StaticSegment("source"), ParamSegment("source_id"), StaticSegment("sentences"))
                        view=SourceSentences
                    />
                    <Route
                        path=(StaticSegment("sentence"), ParamSegment("sentence_id"))
                        view=Sentence
                    />
                    <Route
                        path=(StaticSegment("deck"), StaticSegment("new"))
                        view=DeckNew
                    />
                    <Route
                        path=(StaticSegment("deck"), ParamSegment("deck_id"))
                        view=Deck
                    />
                    <Route
                        path=StaticSegment("ignored-words")
                        view=IgnoredWords
                    />
                    <Route
                        path=StaticSegment("login")
                        view=Login
                    />
                    <Route
                        path=StaticSegment("register")
                        view=Register
                    />
                </FlatRoutes>
            </main>
        </Router>
    }
}
