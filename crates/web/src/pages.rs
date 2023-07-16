//! Top level pages.

use crate::{
    components::{analysis::*, *},
    context::{get_client, get_session},
    error::{WebError, WebResult},
    utils,
};
use lbr_api::response as res;
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

    let other_view = move || {
        if get_session(cx).logged_in().unwrap_or_default() {
            Some({
                view! { cx,
                    <h2 class="subtitle is-6 has-text-weight-bold">"Other"</h2>
                    <A href=format!("/ignored-words")>"Ignored words"</A>
                }
            })
        } else {
            None
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
            <div class="column">
                {other_view}
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
                <label class="label">
                    "Source name"
                    <input class="input" node_ref=name_ref type="text"/>
                </label>
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
    source_id: i32,
}
#[component]
pub fn Source(cx: Scope) -> impl IntoView {
    let SourceParams { source_id } = utils::params(cx)?;
    tracing::info!("Rendering Source {source_id}");

    // resources
    let source_res = utils::logged_in_resource!(cx, get_source_details(source_id));

    // actions
    let name_ref = leptos::create_node_ref::<Input>(cx);
    let (update_result_message, set_update_result_message) =
        leptos::create_signal(cx, ().into_view(cx));
    let update_act = leptos::create_action(cx, move |&()| {
        let name = name_ref().unwrap().value();
        let client = get_client(cx);
        async move {
            client.update_source(source_id, &name).await?;
            source_res.refetch();
            set_update_result_message(view! { cx, <div>"Updated source!"</div> }.into_view(cx));
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
        // TODO: make the user type the name of the source
        let confirmed =
            leptos::window().confirm_with_message("Are you sure you want to delete this source? Doing so will delete all the sentences associated with this source");
        let client = get_client(cx);
        async move {
            let confirmed = confirmed?;
            let view = if confirmed {
                client.delete_source(source_id).await?;
                view! { cx, <Redirect path="/" /> }.into_view(cx)
            } else {
                ().into_view(cx)
            };
            WebResult::Ok(view)
        }
    });

    // source
    let source_content = move |source: res::SourceDetails| {
        let href = format!("/source/{source_id}/add-sentences");
        let sentence_list = source
            .sentences
            .into_iter()
            .map(|sentence| {
                view! { cx,
                    <li>
                        <A href=format!("/source/{source_id}/sentence/{}", sentence.id)>
                            {sentence.sentence}
                        </A>
                    </li>
                }
            })
            .collect_view(cx);
        view! { cx,
            <h2 class="subtitle">{format!("Viewing source {}", source.name)}</h2>
            <div class="block">
                <A href>"Add sentences"</A>
            </div>
            <div class="block">
                <h3 class="subtitle">"Sentences"</h3>
                <div class="content">
                    <ul>
                        {sentence_list}
                    </ul>
                </div>
            </div>
            <div class="block">
                <h3 class="subtitle">"Edit"</h3>
                <form>
                    <label class="label">
                        "Name"
                        <input class="input" type="text" value=source.name node_ref=name_ref/>
                    </label>
                    <button class="button" type="submit" on:click=move |ev| {
                        ev.prevent_default();
                        update_act.dispatch(());
                    }>
                        "Update"
                    </button>
                    <ActionView action=update_act/>
                    {update_result_message}
                </form>
            </div>
            <div class="block">
                <button class="button is-danger" on:click=move |_ev| delete_act.dispatch(())>
                    "Delete source"
                </button>
                <ActionView action=delete_act/>
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
    source_id: i32,
}
#[component]
pub fn SourceAddSentences(cx: Scope) -> impl IntoView {
    let SourceAddSentencesParams { source_id } = utils::params(cx)?;
    tracing::info!("Rendering SourceAddSentences {source_id}");

    let analyse_textarea_ref = leptos::create_node_ref::<Textarea>(cx);
    let analyse_act = leptos::create_action(cx, move |&()| {
        let textarea_val = analyse_textarea_ref().unwrap().value();
        let client = get_client(cx);
        async move { client.segment_paragraph(source_id, &textarea_val).await }
    });

    // source
    let source_res = utils::logged_in_resource!(cx, get_source(source_id));
    let source_content = move |source: res::Source| {
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
    let source_view = move |source: Option<res::Source>| match source {
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
            view! { cx, <div>"Analysing..."</div> }.into_view(cx)
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
        <LoginGuard require_login=true>
            <div class="block">
                <ResourceView resource=source_res view=source_view/>
            </div>
            <div class="block">
                {analysis}
            </div>
        </LoginGuard>
    };
    WebResult::Ok(view)
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct SourceSentenceParams {
    source_id: i32,
    sentence_id: i32,
}
#[component]
pub fn SourceSentence(cx: Scope) -> impl IntoView {
    let SourceSentenceParams {
        source_id,
        sentence_id,
    } = utils::params(cx)?;
    tracing::info!("Rendering Sentence {source_id}");

    let sentence_res = utils::logged_in_resource!(cx, get_sentence(sentence_id));

    let reanalyse_act = leptos::create_action(cx, move |&()| {
        let client = get_client(cx);
        async move { client.segment_sentence(sentence_id).await }
    });
    let delete_act = leptos::create_action(cx, move |&()| {
        let confirmed =
            leptos::window().confirm_with_message("Are you sure you want to delete this sentence?");
        let client = get_client(cx);
        async move {
            let confirmed = confirmed?;
            let view = if confirmed {
                client.delete_sentence(sentence_id).await?;
                Some(view! { cx, <Redirect path={format!("/source/{source_id}")} /> })
            } else {
                None
            };
            WebResult::Ok(view)
        }
    });

    // analysis
    let analysis_content = move |segmented_sentence: res::SegmentedSentence| {
        view! { cx,
            <SegmentedSentenceView source_id sentence_id=Some(sentence_id) segmented_sentence />
        }
    };
    let analysis_view =
        move |segmented: Option<res::SegmentedSentence>| segmented.map(analysis_content);
    let analysis = move || {
        let view = if reanalyse_act.pending().get() {
            view! { cx, <div>"Analysing..."</div> }.into_view(cx)
        } else {
            let segmented = reanalyse_act.value().get().transpose()?;
            analysis_view(segmented).into_view(cx)
        };
        WebResult::Ok(view)
    };
    let analysis = move || {
        view! { cx,
            <ErrorBoundary fallback={utils::errors_fallback}>
                {analysis}
            </ErrorBoundary>
        }
    };

    // sentence
    let sentence_content = move |sentence: res::SentenceDetails| {
        let mut words = sentence.words;
        words.sort_by_key(|v| v.idx_start);
        let words = words
            .into_iter()
            .map(|sw| {
                let word =
                    sentence.sentence[sw.idx_start as usize..sw.idx_end as usize].to_string();
                let translations = sw.translations.join(", ");
                if let Some(reading) = sw.reading {
                    view! { cx,
                        <li>
                            <div>{format!("{word} ({reading})")}</div>
                            <div>{translations}</div>
                        </li>
                    }
                } else {
                    view! { cx,
                        <li>
                            <div>{word}</div>
                            <div>{translations}</div>
                        </li>
                    }
                }
            })
            .collect_view(cx);
        view! { cx,
            <div class="block">
                <div>{sentence.sentence}</div>
            </div>
            <div class="block">
                <button class="button is-primary" on:click=move |_ev| reanalyse_act.dispatch(())>
                    "Reanalyse"
                </button>
            </div>
            {analysis}
            <div class="block">
                <h3 class="subtitle">"Words"</h3>
                <div class="content">
                    <ul>
                        {words}
                    </ul>
                </div>
            </div>
            <div class="block">
                <button class="button is-danger" on:click=move |_ev| delete_act.dispatch(())>
                    "Delete sentence"
                </button>
                <ActionView action=delete_act/>
            </div>
        }
    };
    let sentence_view = move |sentence: Option<res::SentenceDetails>| match sentence {
        Some(sentence) => sentence_content(sentence).into_view(cx),
        None => view! { cx, <div>"Loading sentence..."</div> }.into_view(cx),
    };

    let view = view! { cx,
        <LoginGuard require_login=true>
            <h2 class="subtitle">"Sentence"</h2>
            <ResourceView resource=sentence_res view=sentence_view/>
        </LoginGuard>
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
        <LoginGuard require_login=true>
            <h2 class="subtitle">"Adding new source"</h2>
            <form>
                <label class="label">
                    "Deck name"
                    <input class="input" node_ref=name_ref type="text"/>
                </label>
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
        </LoginGuard>
    }
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct DeckParams {
    deck_id: i32,
}
#[component]
pub fn Deck(cx: Scope) -> impl IntoView {
    let DeckParams { deck_id } = utils::params(cx)?;
    tracing::info!("Rendering Deck {deck_id}");

    // resources
    let deck_res = utils::logged_in_resource!(cx, get_deck(deck_id));
    let sources_res = utils::logged_in_resource!(cx, get_sources());

    // actions
    let name_ref = leptos::create_node_ref::<Input>(cx);
    let (source_checkbox_refs, set_source_checkbox_refs) =
        leptos::create_signal(cx, Vec::<(i32, NodeRef<Input>)>::new());
    let (update_result_message, set_update_result_message) =
        leptos::create_signal(cx, (().into_view(cx), None::<TimeoutHandle>));
    let update_act = leptos::create_action(cx, move |&()| {
        let client = get_client(cx);
        let name = name_ref().unwrap().value();
        let mut included_sources = Vec::new();
        for (id, node_ref) in source_checkbox_refs() {
            let include = node_ref().map(|r| r.checked()).unwrap_or_default();
            if include {
                included_sources.push(id);
            }
        }
        async move {
            client
                .update_deck(deck_id, &name, &included_sources)
                .await?;
            deck_res.refetch();
            if let Some(handle) = update_result_message.get().1 {
                handle.clear();
            }
            let handle = leptos::set_timeout_with_handle(
                move || {
                    set_update_result_message((().into_view(cx), None));
                },
                Duration::from_secs(4),
            )
            .expect("Failed to set timeout");
            set_update_result_message((
                view! { cx, <div>"Updated deck!"</div> }.into_view(cx),
                Some(handle),
            ));

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
                client.delete_deck(deck_id).await?;
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
        let generate_deck_url = client.generate_deck_url(deck_id, &filename);

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
            <h2 class="subtitle">{format!("Viewing deck {}", deck.name)}</h2>
            <div class="block">
                <a href=generate_deck_url download=filename class="button is-primary">
                    "Generate"
                </a>
            </div>
            <div class="block">
                <h3 class="subtitle">"Edit"</h3>
                <form>
                    <label class="label">
                        "Name"
                        <input class="input" value=deck.name node_ref=name_ref/>
                    </label>
                    <label class="label" for="sources-list">
                        "Included sources"
                    </label>
                    <div id="sources-list" class="content">
                        <ul>
                            {sources_list}
                        </ul>
                    </div>
                    <button class="button" type="submit" on:click=move |ev| {
                        ev.prevent_default();
                        update_act.dispatch(());
                    }>
                        "Update"
                    </button>
                    <ActionView action=update_act/>
                    {move || update_result_message().0}
                </form>
            </div>
            <div class="block">
                <button class="button is-danger" on:click=move |_ev| delete_act.dispatch(())>
                    "Delete deck"
                </button>
                <ActionView action=delete_act/>
            </div>
        }
        .into_view(cx)
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
        </LoginGuard>
    }
    .into_view(cx);
    WebResult::Ok(view)
}

#[component]
pub fn IgnoredWords(cx: Scope) -> impl IntoView {
    tracing::info!("Rendering IgnoredWords");

    let delete_act = leptos::create_action(cx, move |word_id: &i32| {
        let confirmed = leptos::window()
            .confirm_with_message("Are you sure you want to delete this ignored word?");
        let word_id = *word_id;
        let client = get_client(cx);
        async move {
            if confirmed? {
                client.delete_ignored_word(word_id).await?;
            }
            Ok(())
        }
    });

    let ignored_words_res = utils::logged_in_resource!(cx, get_ignored_words());
    let ignored_words_content = move |mut ignored_words: Vec<res::IgnoredWord>| {
        if ignored_words.is_empty() {
            return view! { cx, <div>"No ignored words"</div> }.into_view(cx);
        }
        ignored_words.sort_by_key(|iw| iw.word_id);
        let ignored_words = ignored_words
            .into_iter()
            .map(|iw| {
                let translations = iw.translations.join(", ");
                let written_forms = iw
                    .written_forms
                    .into_iter()
                    .map(|wf| {
                        let readings = wf.readings.join(", ");
                        let contents = if readings.is_empty() {
                            wf.written_form
                        } else {
                            format!("{} ({})", wf.written_form, readings)
                        };
                        view! { cx,
                            <li>{contents}</li>
                        }
                    })
                    .collect_view(cx);
                view! { cx,
                    <div class="column">
                        <div class="box">
                            <div>
                                <span class="has-text-weight-bold">"Word id"</span>
                                ": "
                                {iw.word_id}
                            </div>
                            <div>
                                <span class="has-text-weight-bold">"Translations"</span>
                                ": "
                                {translations}
                            </div>
                            <div>
                                <div>
                                    <span class="has-text-weight-bold">"Written forms"</span>
                                    ": "
                                </div>
                                <ul>
                                    {written_forms}
                                </ul>
                            </div>
                            <button class="button is-danger mt-2" on:click=move |_ev| delete_act.dispatch(iw.word_id)>"Delete ignored word"</button>
                        </div>
                    </div>
                }
            })
            .collect_view(cx);
        view! { cx,
            <div class="columns is-flex-wrap-wrap">
                {ignored_words}
            </div>
        }
        .into_view(cx)
    };
    let ignored_words_view = move |ignored_words: Option<Vec<res::IgnoredWord>>| match ignored_words
    {
        Some(ignored_words) => ignored_words_content(ignored_words).into_view(cx),
        None => view! { cx, <div>"Loading..."</div> }.into_view(cx),
    };

    view! { cx,
        <LoginGuard require_login=true>
            <h2 class="subtitle">"Ignored words"</h2>
            <ActionView action=delete_act/>
            <ResourceView resource=ignored_words_res view=ignored_words_view/>
        </LoginGuard>
    }
}

#[component]
pub fn Login(cx: Scope) -> impl IntoView {
    tracing::info!("Rendering Login");

    let redirect = move || {
        leptos_router::use_query_map(cx)
            .get()
            .get("redirect")
            .map(String::to_string)
            .unwrap_or_else(|| "/".to_string())
    };

    let logged_in = move || get_session(cx).logged_in();

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
            let view = move || view! { cx, <Redirect path=redirect() /> };
            WebResult::Ok(view)
        }
    });

    move || {
        if logged_in()? {
            Some(
                view! { cx,
                    <Redirect path=redirect() />
                }
                .into_view(cx),
            )
        } else {
            Some(
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
                .into_view(cx),
            )
        }
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
