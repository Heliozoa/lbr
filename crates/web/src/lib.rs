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
            <ErrorBoundary fallback>
                <Content/>
            </ErrorBoundary>
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
                </AnimatedRoutes>
            </main>
        </Router>
    }
}
