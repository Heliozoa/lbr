//! Custom components.

pub mod analysis;

use crate::{
    context::{get_client, get_session},
    error::WebResult,
    utils,
};
use lbr_api::response::*;
use leptos::prelude::*;
use leptos_router::components::*;
use send_wrapper::SendWrapper;

#[component]
pub fn Navbar() -> impl IntoView {
    let logout_action = Action::new(move |()| {
        let client = get_client();
        async move {
            SendWrapper::new(client.logout()).await?;
            WebResult::Ok(true)
        }
    });

    let navbar_links = move || {
        let view = if get_session().logged_in()? {
            view! {
                <span class="is-flex is-flex-grow-1"></span>
                <button class="button is-link p-3" on:click=move |_ev| { logout_action.dispatch(()); }>"Log out"</button>
            }
            .into_any()
        } else {
            view! {
                <span class="is-flex is-flex-grow-1"></span>
                // class="p-3"
                <A exact=true href="/register">"Register"</A>
                <A exact=true href="/login">"Log in"</A>
            }
            .into_any()
        };
        Some(view)
    };

    view! {
        <nav class="navbar is-flex is-vcentered">
            <A exact=true href="/">"Home"</A>
            {navbar_links}
        </nav>
        <ErrorBoundary fallback={utils::errors_fallback}>
            <Suspense fallback={move || ().into_view()}>
                //<div>
                    {move || logout_action.value().get().map(|o| o.unwrap_or_default()).unwrap_or_default().then(|| {
                        view! { <Redirect path="/" /> }
                    })}
                //</div>
            </Suspense>
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
    let pass = Memo::new(move |passed| {
        if passed.copied().flatten().unwrap_or_default()
            || logged_in().map(|li| li == require_login)?
        {
            Some(true)
        } else {
            Some(false)
        }
    });

    move || {
        let view = if pass.get()? {
            children().into_any()
        } else {
            let redirect = if require_login {
                let url = leptos_router::hooks::use_url().get();
                let redirect = url.path();
                format!("/login?redirect={redirect}")
            } else {
                "/".to_string()
            };
            tracing::info!("Redirecting to {redirect}");
            view! { <Redirect path=redirect /> }.into_any()
        };
        Some(view)
    }
}

#[component]
pub fn ResourceView<T, F, V>(resource: Resource<WebResult<Option<T>>>, view: F) -> impl IntoView
where
    T: Clone + 'static + Send + Sync,
    F: Fn(Option<T>) -> V + Copy + 'static + Send + Sync,
    V: IntoView + 'static,
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
    T: 'static + Send + Sync,
    V: IntoView + Clone + 'static + Send + Sync,
{
    view! {
        <ErrorBoundary fallback={utils::errors_fallback}>
            <div>
                {move || action.value().get()}
            </div>
        </ErrorBoundary>
    }
}
