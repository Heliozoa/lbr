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
pub fn Root() -> impl IntoView {
    let fallback = move |errors: RwSignal<Errors>| {
        errors
            .get_untracked()
            .into_iter()
            .map(|(_key, err)| {
                view! { <div>{format!("Unhandled error: {err}")}</div>}
            })
            .collect_view()
    };
    provide_meta_context();

    view! {
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
                <AnimatedRoutes>
                    <Route
                        path=""
                        view=|| view! { <Home/> }
                    />
                    <Route
                        path="source/new"
                        view=|| view! { <SourceNew/>}
                    />
                    <Route
                        path="source/:source_id"
                        view=|| view! { <Source/> }
                    />
                    <Route
                        path="source/:source_id/add-sentences"
                        view=|| view! { <SourceAddSentences/> }
                    />
                    <Route
                        path="source/:source_id/sentences"
                        view=|| view! { <SourceSentences/> }
                    />
                    <Route
                        path="source/:source_id/sentence/:sentence_id"
                        view=|| view! { <SourceSentence/> }
                    />
                    <Route
                        path="deck/new"
                        view=|| view! { <DeckNew/> }
                    />
                    <Route
                        path="deck/:deck_id"
                        view=|| view! { <Deck/> }
                    />
                    <Route
                        path="ignored-words"
                        view=|| view! { <IgnoredWords/> }
                    />
                    <Route
                        path="login"
                        view=|| view! { <Login/> }
                    />
                    <Route
                        path="register"
                        view=|| view! { <Register/> }
                    />
                </AnimatedRoutes>
            </main>
        </Router>
    }
}
