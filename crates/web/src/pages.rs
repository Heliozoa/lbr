//! Top level pages.

use crate::{
    components::{analysis::*, *},
    context::get_client,
    error::{WebError, WebResult},
    utils,
};
use lbr_api::response as res;
use leptos::{
    html::{Input, Textarea},
    *,
};
use leptos_router::*;
use std::time::Duration;

#[component]
pub fn Home(cx: Scope) -> impl IntoView {
    tracing::info!("Rendering Home");

    // sources
    let sources_res = utils::logged_in_resource!(cx, get_sources());
    let sources_content = move |sources| {
        view! { cx,
            <div class="block">
                <SourceList sources/>
            </div>
        }
    };
    let sources_view = move |sources: Option<_>| {
        view! { cx,
            <h2 class="subtitle is-6 has-text-weight-bold">"Sources"</h2>
            <A href="/source/new">"New source"</A>
            {match sources {
                Some(sources) => sources_content(sources).into_view(cx),
                None => utils::loading_fallback(cx, "Loading sources..."),
            }}
        }
    };

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

    view! { cx,
        <h2 class="subtitle">"Welcome to LBR!"</h2>
        <div class="columns">
            <div class="column">
                <ResourceView resource=sources_res view=sources_view/>
            </div>
            <div class="column">
                <ResourceView resource=decks_res view=decks_view/>
            </div>
        </div>
    }
}

#[component]
pub fn SourceNew(cx: Scope) -> impl IntoView {
    tracing::info!("Rendering SourceNew");

    let name_ref = leptos::create_node_ref::<Input>(cx);
    let send = leptos::create_action(cx, move |()| {
        let name = name_ref().unwrap().value();
        let client = get_client(cx);
        async move {
            if name.is_empty() {
                return Err(WebError {
                    message: "Source name cannot be empty".to_string(),
                });
            }
            let id = client.new_source(&name).await?;
            WebResult::Ok(view! { cx, <Redirect path=format!("/source/{id}") /> })
        }
    });

    view! { cx,
        <LoginGuard require_login=true>
            <h2 class="subtitle">"Adding new source"</h2>
            <form>
                <div class="block">
                    <label class="label">
                        "Source name"
                        <input class="input" node_ref=name_ref type="text"/>
                    </label>
                </div>
                <div class="block">
                    <button class="button" type="submit" on:click=move |ev| {
                        ev.prevent_default();
                        send.dispatch(());
                    }>
                        "Save"
                    </button>
                    <ActionView action=send/>
                </div>
            </form>
        </LoginGuard>
    }
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct SourceParams {
    id: i32,
}
#[component]
pub fn Source(cx: Scope) -> impl IntoView {
    let SourceParams { id } = utils::params(cx)?;
    tracing::info!("Rendering Source {id}");

    let delete_act = leptos::create_action(cx, move |&()| {
        // TODO: make the user type the name of the source
        let confirmed =
            leptos::window().confirm_with_message("Are you sure you want to delete this source? Doing so will delete all the sentences associated with this source");
        let client = get_client(cx);
        async move {
            let confirmed = confirmed?;
            let view = if confirmed {
                client.delete_source(id).await?;
                view! { cx, <Redirect path="/" /> }.into_view(cx)
            } else {
                ().into_view(cx)
            };
            WebResult::Ok(view)
        }
    });

    // source
    let source_res = utils::logged_in_resource!(cx, get_source(id));
    let source_content = move |source: res::SourceWithSentences| {
        let href = format!("/source/{id}/add-sentences");
        let sentence_list = source
            .sentences
            .into_iter()
            .map(|sentence| {
                view! { cx,
                    <li>
                        <A href=format!("/sentence/{}", sentence.id)>
                            {sentence.sentence}
                        </A>
                    </li>
                }
            })
            .collect_view(cx);
        view! { cx,
            <h2 class="subtitle">"Viewing source " {source.name}</h2>
            <div class="block">
                <A href>"Add sentences"</A>
            </div>
            <div class="block">
                <button class="button is-danger" on:click=move |_ev| delete_act.dispatch(())>
                    "Delete"
                </button>
                <ActionView action=delete_act/>
            </div>
            <div class="block">
                <h3 class="subtitle">"Sentences"</h3>
                <div class="content">
                    <ul>
                        {sentence_list}
                    </ul>
                </div>
            </div>
        }
    };
    let source_view = move |source: Option<_>| match source {
        Some(source) => source_content(source).into_view(cx),
        None => utils::loading_fallback(cx, "Loading source..."),
    };

    let view = view! { cx,
        <LoginGuard require_login=true>
            <ResourceView resource=source_res view=source_view />
        </LoginGuard>
    };
    WebResult::Ok(view)
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct SourceAddSentencesParams {
    id: i32,
}
#[component]
pub fn SourceAddSentences(cx: Scope) -> impl IntoView {
    let SourceAddSentencesParams { id: source_id } = utils::params(cx)?;
    tracing::info!("Rendering SourceAddSentences {source_id}");

    let analyse_textarea_ref = leptos::create_node_ref::<Textarea>(cx);
    let analyse_act = leptos::create_action(cx, move |&()| {
        let textarea_val = analyse_textarea_ref().unwrap().value();
        let client = get_client(cx);
        async move { client.segment_paragraph(source_id, &textarea_val).await }
    });

    // source
    let source_res = utils::logged_in_resource!(cx, get_source(source_id));
    let source_content = move |source: res::SourceWithSentences| {
        view! { cx,
            <h2 class="subtitle">{source.name}</h2>
            <label class="label">
                "Paragraph"
                <textarea class="textarea" node_ref=analyse_textarea_ref/>
            </label>
            <button class="button is-primary mt-1" on:click=move |_ev| analyse_act.dispatch(())>
                "Analyse"
            </button>
        }
    };
    let source_view = move |source: Option<res::SourceWithSentences>| match source {
        Some(source) => source_content(source).into_view(cx),
        None => view! { cx, <div>"Loading source..."</div> }.into_view(cx),
    };

    // analysis
    let analysis_content = move |segmented: Vec<res::SegmentedSentence>| {
        view! { cx, <SegmentedParagraphView source_id=source_id segmented /> }
    };
    let analysis_view = move |segmented: Option<Vec<res::SegmentedSentence>>| match segmented {
        Some(segments) => analysis_content(segments).into_view(cx),
        None => view! { cx, <div>"Nothing analysed yet"</div> }.into_view(cx),
    };
    let analysis = move || {
        let view = if analyse_act.pending().get() {
            view! { cx, <div>"Segmenting..."</div> }.into_view(cx)
        } else {
            let segmented = analyse_act.value().get().transpose()?;
            analysis_view(segmented).into_view(cx)
        };
        WebResult::Ok(view)
    };
    let analysis = view! { cx,
        <ErrorBoundary fallback={utils::errors_fallback}>
            {analysis}
        </ErrorBoundary>
    };

    let view = view! { cx,
        <div class="block">
            <ResourceView resource=source_res view=source_view/>
        </div>
        <div class="block">
            {analysis}
        </div>
    };
    WebResult::Ok(view)
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct SentenceParams {
    id: i32,
}
#[component]
pub fn Sentence(cx: Scope) -> impl IntoView {
    let SentenceParams { id } = utils::params(cx)?;
    tracing::info!("Rendering Sentence {id}");

    // sentence
    let sentence_res = utils::logged_in_resource!(cx, get_sentence(id));
    let sentence_content = move |sentence: res::Sentence| {
        view! { cx,
            <div>{sentence.sentence}</div>
        }
    };
    let sentence_view = move |sentence: Option<res::Sentence>| match sentence {
        Some(sentence) => sentence_content(sentence).into_view(cx),
        None => view! { cx, <div>"Loading sentence..."</div> }.into_view(cx),
    };

    let view = view! { cx,
        <h2 class="subtitle">"Sentence"</h2>
        <ResourceView resource=sentence_res view=sentence_view/>
    };
    WebResult::Ok(view)
}

#[component]
pub fn DeckNew(cx: Scope) -> impl IntoView {
    tracing::info!("Rendering DeckNew");

    let name_ref = leptos::create_node_ref::<Input>(cx);
    let save_act = leptos::create_action(cx, move |&()| {
        let name = name_ref().unwrap().value();
        let client = get_client(cx);
        async move {
            if name.is_empty() {
                return Err(WebError {
                    message: "Deck name cannot be empty!".to_string(),
                });
            }
            let id = client.new_deck(&name).await?;
            WebResult::Ok(view! { cx, <Redirect path=format!("/deck/{id}") /> })
        }
    });

    view! { cx,
        <h2 class="subtitle">"Adding new source"</h2>
        <form>
            <div class="block">
                <label class="label">
                    "Deck name"
                    <input class="input" node_ref=name_ref type="text"/>
                </label>
            </div>
            <div class="block">
                <button class="button" type="submit" on:click=move |ev| {
                    ev.prevent_default();
                    save_act.dispatch(());
                }>
                    "Save"
                </button>
                <ActionView action=save_act/>
            </div>
        </form>
    }
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct DeckParams {
    id: i32,
}
#[component]
pub fn Deck(cx: Scope) -> impl IntoView {
    let DeckParams { id } = utils::params(cx)?;
    tracing::info!("Rendering Deck {id}");

    // resources
    let deck_res = utils::logged_in_resource!(cx, get_deck(id));
    let sources_res = utils::logged_in_resource!(cx, get_sources());

    // actions
    let (source_checkbox_refs, set_source_checkbox_refs) =
        leptos::create_signal(cx, Vec::<(i32, NodeRef<Input>)>::new());
    let (update_result_message, set_update_result_message) =
        leptos::create_signal(cx, ().into_view(cx));
    let update_sources_act = leptos::create_action(cx, move |&()| {
        let client = get_client(cx);
        let mut included_sources = Vec::new();
        for (id, node_ref) in source_checkbox_refs() {
            let include = node_ref().map(|r| r.checked()).unwrap_or_default();
            if include {
                included_sources.push(id);
            }
        }
        async move {
            client.update_deck_sources(id, &included_sources).await?;
            set_update_result_message(
                view! { cx, <div>"Updated included sources!"</div> }.into_view(cx),
            );
            leptos::set_timeout(
                move || {
                    set_update_result_message(().into_view(cx));
                },
                Duration::from_secs(4),
            );
            WebResult::Ok(())
        }
    });
    let delete_act = leptos::create_action(cx, move |&()| {
        let confirmed =
            leptos::window().confirm_with_message("Are you sure you want to delete this deck?");
        let client = get_client(cx);
        async move {
            let confirmed = confirmed?;
            let view = if confirmed {
                client.delete_deck(id).await?;
                view! { cx, <Redirect path="/" /> }.into_view(cx)
            } else {
                ().into_view(cx)
            };
            WebResult::Ok(view)
        }
    });

    // views
    let deck_sources_content = move |deck: res::DeckDetails, sources: Vec<res::Source>| {
        let client = get_client(cx);
        let today = chrono::Utc::now().date_naive();
        let filename = format!("lbr-{}-{}.apkg", deck.name, today);
        let generate_deck_url = client.generate_deck_url(id, &filename);

        let mut refs = vec![];
        let sources_list = sources
            .into_iter()
            .map(|s| {
                let checked = deck.sources.contains(&s.id);
                let include_ref = leptos::create_node_ref::<Input>(cx);
                refs.push((s.id, include_ref));
                view! { cx,
                    <li>
                        <div>{s.name}</div>
                        <label class="checkbox">
                            <input class="mr-1" type="checkbox" checked=checked node_ref=include_ref/>
                            "Include"
                        </label>
                    </li>
                }
            })
            .collect_view(cx);
        set_source_checkbox_refs(refs);

        view! { cx,
            <h2 class="subtitle">"Viewing deck " {deck.name}</h2>
            <div class="block">
                <a href=generate_deck_url download=filename class="button is-primary" >"Generate"</a>
            </div>
            <div class="block">
                <button class="button is-danger" on:click=move |_ev| delete_act.dispatch(())>"Delete"</button>
                    <ErrorBoundary fallback={utils::errors_fallback}>
                        {move || delete_act.value()}
                    </ErrorBoundary>
            </div>

            <h2>"Included sources"</h2>
            <form>
                <div class="content">
                    <ul>
                        {sources_list}
                    </ul>
                </div>
                <button class="button" type="submit" on:click=move |ev| {
                    ev.prevent_default();
                    update_sources_act.dispatch(());
                }>
                    "Update included sources"
                </button>
            </form>
        }.into_view(cx)
    };
    let deck_sources_view = move |deck: Option<res::DeckDetails>,
                                  sources: Option<Vec<res::Source>>| {
        let view = match (deck, sources) {
            (Some(d), Some(s)) => deck_sources_content(d, s).into_view(cx),
            (None, _) => view! { cx, <div>"Loading deck..."</div> }.into_view(cx),
            (_, None) => view! { cx, <div>"Loading sources..."</div> }.into_view(cx),
        };
        view
    };
    let deck_sources = view! { cx,
        <Suspense fallback={move || deck_sources_view(None, None)}>
            <ErrorBoundary fallback={utils::errors_fallback}>
                {move || {
                    let deck = utils::untangle!(cx, deck_res);
                    let sources = utils::untangle!(cx, sources_res);
                    WebResult::Ok(Some(deck_sources_view(deck, sources)))
                }}
            </ErrorBoundary>
        </Suspense>
    };

    let view = view! { cx,
        <LoginGuard require_login=true>
            {deck_sources}
            <div>{move || update_result_message()}</div>
            <ActionView action=update_sources_act/>
            <ActionView action=delete_act/>
        </LoginGuard>
    };
    WebResult::Ok(view)
}

#[component]
pub fn Login(cx: Scope) -> impl IntoView {
    tracing::info!("Rendering Login");

    let redirect = move || {
        leptos_router::use_query_map(cx)
            .get_untracked()
            .get("redirect")
            .map(String::to_string)
            .unwrap_or_else(|| "/".to_string())
    };

    // form
    let email_ref = leptos::create_node_ref::<Input>(cx);
    let password_ref = leptos::create_node_ref::<Input>(cx);
    let submission_act = leptos::create_action(cx, move |&()| {
        let email = email_ref().unwrap().value();
        let password = password_ref().unwrap().value();
        let client = get_client(cx);
        async move {
            if email.is_empty() {
                return Err(WebError::new("Email cannot be empty"));
            }
            if password.is_empty() {
                return Err(WebError::new("Password cannot be empty"));
            }
            client.login(email.as_str(), password.as_str()).await?;
            let redirect = redirect();
            WebResult::Ok(view! { cx, <Redirect path=redirect /> }.into_view(cx))
        }
    });

    view! { cx,
        <h2 class="subtitle">"Login"</h2>
        <form>
            <label class="label">
                "Email"
                <input class="input" node_ref=email_ref/>
            </label>
            <label class="label">
                "Password"
                <input class="input" node_ref=password_ref/>
            </label>
            <button class="button" type="submit" on:click={move |ev| {
                ev.prevent_default();
                submission_act.dispatch(());
            }}>
                "Submit"
            </button>
            <ErrorBoundary fallback={utils::errors_fallback}>
                {move || submission_act.value()}
            </ErrorBoundary>
        </form>
    }
}

#[component]
pub fn Register(cx: Scope) -> impl IntoView {
    tracing::info!("Rendering Register");

    // form
    let email_ref = leptos::create_node_ref::<Input>(cx);
    let password_ref = leptos::create_node_ref::<Input>(cx);
    let submission_act = leptos::create_action(cx, move |&()| {
        let email = email_ref().unwrap().value();
        let password = password_ref().unwrap().value();
        let client = get_client(cx);
        async move {
            if email.is_empty() {
                return Err(WebError::new("Email cannot be empty"));
            }
            if password.is_empty() {
                return Err(WebError::new("Password cannot be empty"));
            }
            client.register(&email, &password).await?;
            WebResult::Ok(move || view! { cx, <Redirect path="/login" /> })
        }
    });

    view! { cx,
        <LoginGuard require_login=false>
            <h2 class="subtitle">"Register"</h2>
            <form>
                <label class="label">
                    "Email"
                    <input class="input" node_ref=email_ref/>
                </label>
                <label class="label">
                    "Password"
                    <input class="input" node_ref=password_ref/>
                </label>
                <button class="button" type="submit" on:click={move |ev| {
                    ev.prevent_default();
                    submission_act.dispatch(())
                }}>
                    "Submit"
                </button>
                <ActionView action=submission_act/>
            </form>
        </LoginGuard>
    }
}
