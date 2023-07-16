pub mod components;
pub mod context;
pub mod error;
pub mod pages;
pub mod utils;

use components::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use pages::*;

/// Wraps the content in a basic layout and a final fallback error boundary which should never actually trigger
#[component]
pub fn Root(cx: Scope) -> impl IntoView {
    let fallback = move |cx: Scope, errors: RwSignal<Errors>| {
        errors
            .get_untracked()
            .into_iter()
            .map(|(_key, err)| {
                view! {cx, <div>{format!("Unhandled error: {err}")}</div>}
            })
            .collect_view(cx)
    };
    provide_meta_context(cx);

    view! { cx,
            <Stylesheet id="lbr" href="/pkg/lbr.css"/>
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
                        " - Powered by "
                        <a href="https://github.com/tshatrov/ichiran">"ichiran"</a>
                        " / "
                        <a href="https://github.com/tokio-rs/axum">"axum"</a>
                        " / "
                        <a href="https://github.com/diesel-rs/diesel">"diesel"</a>
                        " / "
                        <a href="https://github.com/leptos-rs/leptos/">"Leptos"</a>
                        " / "
                        <a href="https://bulma.io/">"Bulma"</a>
                        " and more"
                    </div>
                </footer>
            </div>
    }
}

/// Contains the navbar and router
#[component]
pub fn Content(cx: Scope) -> impl IntoView {
    view! { cx,
        <Router>
            <Navbar/>
            <main>
                <h1 class="title">"LBR"</h1>
                <AnimatedRoutes>
                    <Route
                        path=""
                        view=|cx| view! { cx, <Home/> }
                    />
                    <Route
                        path="source/new"
                        view=|cx| view! { cx, <SourceNew/>}
                    />
                    <Route
                        path="source/:source_id"
                        view=|cx| view! { cx, <Source/> }
                    />
                    <Route
                        path="source/:source_id/add-sentences"
                        view=|cx| view! { cx, <SourceAddSentences/> }
                    />
                    <Route
                        path="source/:source_id/sentence/:sentence_id"
                        view=|cx| view! { cx, <SourceSentence/> }
                    />
                    <Route
                        path="deck/new"
                        view=|cx| view! { cx, <DeckNew/> }
                    />
                    <Route
                        path="deck/:deck_id"
                        view=|cx| view! { cx, <Deck/> }
                    />
                    <Route
                        path="ignored-words"
                        view=|cx| view! { cx, <IgnoredWords/> }
                    />
                    <Route
                        path="login"
                        view=|cx| view! { cx, <Login/> }
                    />
                    <Route
                        path="register"
                        view=|cx| view! { cx, <Register/> }
                    />
                </AnimatedRoutes>
            </main>
        </Router>
    }
}
