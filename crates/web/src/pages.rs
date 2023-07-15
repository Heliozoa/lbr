//! Top level pages.

use crate::{
    components::*,
    context::{get_client, get_session},
    error::{WebError, WebResult},
    utils,
};
use leptos::{
    html::{Input, Textarea},
    leptos_dom::helpers::TimeoutHandle,
    *,
};
use leptos_router::*;
use std::time::Duration;

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    tracing::info!("Rendering Home");

    // decks
    let decks_res = utils::logged_in_resource!(cx, get_decks());
    let decks_content = move |decks| {
        view! { cx,
            <div class="block">
                <DeckList decks/>
            </div>
        }
    };
    let decks_view = move |decks: Option<_>| {
        view! { cx,
            <h2 class="subtitle is-6 has-text-weight-bold">"Decks"</h2>
            <A href="/deck/new">"New deck"</A>
            {match decks {
                Some(decks) => decks_content(decks).into_view(cx),
                None => utils::loading_fallback(cx, "Loading decks..."),
            }}
        }
    };

    let other_view = move || {
        if get_session(cx).logged_in().unwrap_or_default() {
            tracing::info!("logged in");
            Some(view! { cx,
                <div>
                    "hi!"
                </div>
            })
        } else {
            None
        }
    };

    view! { cx,
        //<h2 class="subtitle">"Welcome to LBR!"</h2>
        //<div class="columns">
            //<div id="col-1" class="column">
                //<ResourceView resource=sources_res view=sources_view/>
            //</div>
            <div>"above column"</div>
            <ResourceView resource=decks_res view=decks_view/>
            <div>"below column"</div>
            {other_view}
        //</div>
    }
}
