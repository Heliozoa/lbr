//! Custom components.

pub mod analysis;

use crate::{
    context::{get_client, get_session},
    error::WebResult,
    utils,
};
use lbr_api::response::*;
use leptos::*;
use leptos_router::*;
use std::cell::RefCell;

#[component]
pub fn Navbar(cx: Scope) -> impl IntoView {
    let logout_action = leptos::create_action(cx, move |()| {
        let client = get_client(cx);
        async move {
            client.logout().await?;
            WebResult::Ok(view! { cx, <Redirect path="/" /> }.into_view(cx))
        }
    });

    let navbar_links = move || {
        let view = if get_session(cx).logged_in()? {
            view! { cx,
                <span class="is-flex is-flex-grow-1"></span>
                <button class="button is-link p-3" on:click=move |_ev| logout_action.dispatch(())>"Logout"</button>
            }
            .into_view(cx)
        } else {
            view! { cx,
                <span class="is-flex is-flex-grow-1"></span>
                <A class="p-3" exact=true href="/register">"Register"</A>
                <A class="p-3" exact=true href="/login">"Login"</A>
            }
            .into_view(cx)
        };
        Some(view)
    };

    view! { cx,
        <nav class="navbar is-flex is-vcentered">
            <A class="p-3" exact=true href="/">"Home"</A>
            {navbar_links}
        </nav>
        <ErrorBoundary fallback={utils::errors_fallback}>
            <div>{move || logout_action.value()}</div>
        </ErrorBoundary>
    }
}

#[component]
pub fn SourceList(cx: Scope, sources: Vec<Source>) -> impl IntoView {
    let sources = sources
        .into_iter()
        .map(|source| {
            let href = format!("source/{}", source.id);
            view! { cx,
                <li>
                    <A href>{source.name}</A>
                </li>
            }
        })
        .collect_view(cx);
    view! { cx,
        <div class="content">
            <ul>
                {sources}
            </ul>
        </div>
    }
}

#[component]
pub fn DeckList(cx: Scope, decks: Vec<Deck>) -> impl IntoView {
    let decks = decks
        .into_iter()
        .map(|deck| {
            let href = format!("deck/{}", deck.id);
            view! { cx,
                <li>
                    <A href>{deck.name}</A>
                </li>
            }
        })
        .collect_view(cx);
    view! { cx,
    <div class="content">
        <ul>
            {decks}
        </ul>
    </div>}
}

#[component]
pub fn LoginGuard(cx: Scope, children: Children, require_login: bool) -> impl IntoView {
    let logged_in = move || get_session(cx).logged_in();
    let pass = move || logged_in().map(|li| li == require_login);

    let children = RefCell::new(Some(children));
    move || {
        let view = if pass()? {
            children.borrow_mut().take().unwrap()(cx).into_view(cx)
        } else {
            let redirect = if require_login {
                let redirect = leptos_router::use_route(cx).path();
                format!("/login?redirect={redirect}")
            } else {
                "/".to_string()
            };
            view! { cx, <Redirect path=redirect /> }.into_view(cx)
        };
        Some(view)
    }
}

#[component]
pub fn ResourceView<T, F, V>(
    cx: Scope,
    resource: Resource<Option<bool>, WebResult<Option<T>>>,
    view: F,
) -> impl IntoView
where
    T: Clone + 'static,
    F: Fn(Option<T>) -> V + Copy + 'static,
    V: IntoView,
{
    let resource_view = move || match resource.read(cx) {
        Some(Ok(Some(res))) => Ok(Some(view(Some(res)).into_view(cx))),
        Some(Ok(None)) => Ok(None),
        Some(Err(err)) => Err(err),
        None => Ok(Some(view(None).into_view(cx))),
    };
    let resource_view = leptos::store_value(cx, resource_view);
    let wrapped_view = view! { cx,
        <Suspense fallback={move || view(None)}>
            <ErrorBoundary fallback={utils::errors_fallback}>
                {resource_view}
            </ErrorBoundary>
        </Suspense>
    };
    WebResult::Ok(wrapped_view)
}

#[component]
pub fn ActionView<V>(cx: Scope, action: Action<(), WebResult<V>>) -> impl IntoView
where
    V: IntoView + Clone + 'static,
{
    view! { cx,
        <ErrorBoundary fallback={utils::errors_fallback}>
            <div>{move || action.value()}</div>
        </ErrorBoundary>
    }
}
