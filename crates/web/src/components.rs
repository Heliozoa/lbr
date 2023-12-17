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

#[component]
pub fn Navbar() -> impl IntoView {
    let logout_action = leptos::create_action(move |()| {
        let client = get_client();
        async move {
            client.logout().await?;
            WebResult::Ok(view! { <Redirect path="/" /> }.into_view())
        }
    });

    let navbar_links = move || {
        let view = if get_session().logged_in()? {
            view! {
                <span class="is-flex is-flex-grow-1"></span>
                <button class="button is-link p-3" on:click=move |_ev| logout_action.dispatch(())>"Logout"</button>
            }
            .into_view()
        } else {
            view! {
                <span class="is-flex is-flex-grow-1"></span>
                <A class="p-3" exact=true href="/register">"Register"</A>
                <A class="p-3" exact=true href="/login">"Login"</A>
            }
            .into_view()
        };
        Some(view)
    };

    view! {
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
pub fn SourceList(sources: Vec<Source>) -> impl IntoView {
    let sources = sources
        .into_iter()
        .map(|source| {
            let href = format!("source/{}", source.id);
            view! {
                <li>
                    <A href>{source.name}</A>
                </li>
            }
        })
        .collect_view();
    view! {
        <div class="content">
            <ul>
                {sources}
            </ul>
        </div>
    }
}

#[component]
pub fn DeckList(decks: Vec<Deck>) -> impl IntoView {
    let decks = decks
        .into_iter()
        .map(|deck| {
            let href = format!("deck/{}", deck.id);
            view! {
                <li>
                    <A href>{deck.name}</A>
                </li>
            }
        })
        .collect_view();
    view! {
    <div class="content">
        <ul>
            {decks}
        </ul>
    </div>}
}

#[component]
pub fn LoginGuard(children: ChildrenFn, require_login: bool) -> impl IntoView {
    let logged_in = move || get_session().logged_in();
    let pass = leptos::create_memo(move |passed| {
        if passed.copied().flatten().unwrap_or_default() {
            Some(true)
        } else if logged_in().map(|li| li == require_login)? {
            Some(true)
        } else {
            Some(false)
        }
    });

    move || {
        let view = if pass()? {
            children().into_view()
        } else {
            let redirect = if require_login {
                let redirect = leptos_router::use_route().path();
                format!("/login?redirect={redirect}")
            } else {
                "/".to_string()
            };
            view! { <Redirect path=redirect /> }.into_view()
        };
        Some(view)
    }
}

#[component]
pub fn ResourceView<T, F, V>(
    resource: Resource<Option<bool>, WebResult<Option<T>>>,
    view: F,
) -> impl IntoView
where
    T: Clone + 'static,
    F: Fn(Option<T>) -> V + Copy + 'static,
    V: IntoView,
{
    let resource_view = move || match resource.get() {
        Some(Ok(Some(res))) => Ok(Some(view(Some(res)).into_view())),
        Some(Ok(None)) => Ok(None),
        Some(Err(err)) => Err(err),
        None => Ok(Some(view(None).into_view())),
    };
    let wrapped_view = view! {
        <Suspense fallback={move || view(None)}>
            <ErrorBoundary fallback={utils::errors_fallback}>
                {resource_view}
            </ErrorBoundary>
        </Suspense>
    };
    WebResult::Ok(wrapped_view)
}

#[component]
pub fn ActionView<T, V>(action: Action<T, WebResult<V>>) -> impl IntoView
where
    T: 'static,
    V: IntoView + Clone + 'static,
{
    view! {
        <ErrorBoundary fallback={utils::errors_fallback}>
            <div>{move || action.value()}</div>
        </ErrorBoundary>
    }
}
