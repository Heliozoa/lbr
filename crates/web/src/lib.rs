pub mod context;
pub mod error;

use crate::{
    context::{get_session, Client},
    error::WebResult,
};
use leptos::*;
use leptos_router::*;

#[component]
pub fn Root(cx: Scope) -> impl IntoView {
    view! { cx,
        <Router>
            <Routes>
                <Route
                    path=""
                    view=|cx| view! { cx, <Home/> }
                />
            </Routes>
        </Router>
    }
}

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    let decks_res = leptos::create_resource(
        cx,
        move || get_session(cx).logged_in(),
        move |logged_in| async move {
            let data = match logged_in {
                Some(true) => {
                    let data = Client.get_decks().await?;
                    Some(data)
                }
                _ => None,
            };
            WebResult::Ok(data)
        },
    );

    let other_view = move || {
        if get_session(cx).logged_in().unwrap_or_default() {
            view! { cx,
                <div>"true"</div>
            }
        } else {
            view! { cx,
                <div>"false"</div>
            }
        }
    };

    let decks_view = move || match decks_res.read(cx) {
        Some(Ok(Some(res))) => res
            .into_iter()
            .map(|d| view! { cx, <div>{d}</div>})
            .collect_view(cx),
        _ => ().into_view(cx),
    };
    view! { cx,
        <div>"above column"</div>
        <Suspense fallback={move || ()}>
            <ErrorBoundary fallback={|_, _| panic!()}>
                {decks_view}
            </ErrorBoundary>
        </Suspense>
        <div>"below column"</div>
        {other_view}
    }
}
