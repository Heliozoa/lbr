//! Top level pages.

use crate::{
    components::{analysis::*, *},
    context::{get_client, get_session},
    error::{WebError, WebResult},
    utils,
};
use lbr_api::{request as req, response as res};
use leptos::{
    html::{Input, Textarea},
    leptos_dom::helpers::TimeoutHandle,
    prelude::*,
};
use leptos_router::{components::*, params::Params};
use send_wrapper::SendWrapper;
use std::{sync::Arc, time::Duration};

#[component]
pub fn Home() -> impl IntoView {
    tracing::info!("Rendering Home");

    // sources
    let sources_res = utils::logged_in_resource!(get_sources());
    let sources_content = move |mut sources: Vec<res::Source>| {
        sources.sort_unstable_by(|l, r| l.name.cmp(&r.name));
        view! {
            <div class="block">
                <SourceList sources/>
            </div>
        }
        .into_any()
    };
    let sources_view = move |sources: Option<_>| {
        view! {
            <h2 class="subtitle is-6 has-text-weight-bold">"Sources"</h2>
            <A href="/source/new">"New source"</A>
            {match sources {
                Some(sources) => sources_content(sources).into_any(),
                None => utils::loading_fallback("Loading sources...").into_any(),
            }}
        }
        .into_any()
    };

    // decks
    let decks_res = utils::logged_in_resource!(get_decks());
    let decks_content = move |mut decks: Vec<res::Deck>| {
        decks.sort_unstable_by(|l, r| l.name.cmp(&r.name));
        view! {
            <div class="block">
                <DeckList decks/>
            </div>
        }
        .into_any()
    };
    let decks_view = move |decks: Option<_>| {
        view! {
            <h2 class="subtitle is-6 has-text-weight-bold">"Decks"</h2>
            <A href="/deck/new">"New deck"</A>
            {match decks {
                Some(decks) => decks_content(decks).into_any(),
                None => utils::loading_fallback("Loading decks...").into_any(),
            }}
        }
        .into_any()
    };

    let other_view = move || {
        if get_session().logged_in().unwrap_or_default() {
            Some({
                view! {
                    <h2 class="subtitle is-6 has-text-weight-bold">"Other"</h2>
                    <A href="/ignored-words">"Ignored words"</A>
                }
                .into_any()
            })
        } else {
            None
        }
    };

    view! {
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
pub fn SourceNew() -> impl IntoView {
    tracing::info!("Rendering SourceNew");

    let name_ref = NodeRef::<Input>::new();
    let send = Action::new(move |()| async move {
        let name = name_ref.get().expect("failed to get name_ref").value();
        let client = get_client();
        if name.is_empty() {
            return Err(WebError {
                message: "Source name cannot be empty".to_string(),
            });
        }
        let id = SendWrapper::new(client.new_source(&name)).await?;
        WebResult::Ok(view! { <Redirect path=format!("/source/{id}") /> })
    });

    view! {
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
                        "Create source"
                    </button>
                    <ActionView action=send/>
                </div>
            </form>
        </LoginGuard>
    }
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct SourceParams {
    source_id: Option<i32>,
}
#[component]
pub fn Source() -> impl IntoView {
    let SourceParams { source_id } = utils::params()?;
    let source_id = source_id.expect("failed to get source_id");
    tracing::info!("Rendering Source {source_id}");

    // resources
    let source_res = utils::logged_in_resource!(get_source(source_id));

    // actions
    let name_ref = NodeRef::<Input>::new();
    let (update_result_message, set_update_result_message) =
        leptos::prelude::signal((None::<&'static str>, None::<TimeoutHandle>));
    let update_act = Action::new(move |&()| {
        let name = name_ref.get().expect("Failed to get name_ref").value();
        let client = get_client();
        async move {
            SendWrapper::new(client.update_source(source_id, &name)).await?;
            // todo source_res.refetch();
            let handle = leptos::prelude::set_timeout_with_handle(
                move || {
                    set_update_result_message.set((None, None));
                },
                Duration::from_secs(4),
            )
            .ok();
            set_update_result_message.set((Some("Updated source!"), handle));
            WebResult::Ok(())
        }
    });
    let delete_act = Action::new(move |&()| {
        // TODO: make the user type the name of the source
        let confirmed = leptos::prelude::window()
            .confirm_with_message(
                "Are you sure you want to delete this source? \
            Doing so will delete all the sentences associated with this source",
            )
            .map_err(WebError::from_js);
        let client = get_client();
        async move {
            let confirmed = confirmed?;
            let view = if confirmed {
                SendWrapper::new(client.delete_source(source_id)).await?;
                Some(view! { <Redirect path="/" /> })
            } else {
                None
            };
            WebResult::Ok(view)
        }
    });

    // source
    let source_content = move |source: res::Source| {
        let add_sentences_href = format!("/source/{source_id}/add-sentences");
        let sentences_href = format!("/source/{source_id}/sentences");
        view! {
            <h2 class="subtitle">{format!("Viewing source {}", source.name)}</h2>
            <div class="block">
                <A href=add_sentences_href>"Add sentences"</A>
            </div>
            <div class="block">
                <A href=sentences_href>"View sentences"</A>
            </div>
            <div class="block">
                <h3 class="subtitle">"Edit source"</h3>
                <form>
                    <label class="label">
                        "Source name"
                        <input class="input" type="text" value=source.name node_ref=name_ref/>
                    </label>
                    <button class="button" type="submit" on:click=move |ev| {
                        ev.prevent_default();
                        update_act.dispatch(());
                    }>
                        "Update source"
                    </button>
                    <ActionView action=update_act/>
                    {move || update_result_message.get().0}
                </form>
            </div>
            <div class="block">
                <button class="button is-danger" on:click=move |_ev| { delete_act.dispatch(()); }>
                    "Delete source"
                </button>
                <ActionView action=delete_act/>
            </div>
        }
    };
    let source_view = move |source: Option<_>| match source {
        Some(source) => source_content(source).into_any(),
        None => utils::loading_fallback("Loading source...").into_any(),
    };

    let view = view! {
        <LoginGuard require_login=true>
            <ResourceView resource=source_res view=source_view />
        </LoginGuard>
    };
    WebResult::Ok(view)
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct SentencesParams {
    source_id: Option<i32>,
}
#[component]
pub fn SourceSentences() -> impl IntoView {
    let SentencesParams { source_id } = utils::params()?;
    let source_id = source_id.expect("failed to get source_id");
    tracing::info!("Rendering SourceSentences {source_id}");

    // resources
    let source_res = utils::logged_in_resource!(get_source_details(source_id));

    // source
    let sentences = move |sentences: Vec<res::Sentence>| {
        let sentences_list = sentences
            .into_iter()
            .map(|s| {
                view! {
                    <li>
                        <A href=format!("/source/{source_id}/sentence/{}", s.id)>{s.sentence}</A>
                    </li>
                }
            })
            .collect_view();
        view! {
            <div class="content">
                <ul>
                    {sentences_list}
                </ul>
            </div>
        }
    };
    let source_content = move |source: res::SourceDetails| {
        let sentences_view = sentences(source.sentences);
        view! {
            <h2 class="subtitle">{format!("Viewing sentences for source {}", source.name)}</h2>
            <div class="block">
                <h3 class="subtitle">"Sentences"</h3>
                {sentences_view}
            </div>
        }
    };
    let source_view = move |source: Option<_>| match source {
        Some(source) => source_content(source).into_any(),
        None => utils::loading_fallback("Loading source...").into_any(),
    };

    let view = view! {
        <LoginGuard require_login=true>
            <ResourceView resource=source_res view=source_view />
        </LoginGuard>
    };
    WebResult::Ok(view)
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct SourceAddSentencesParams {
    source_id: Option<i32>,
}
#[component]
pub fn SourceAddSentences() -> impl IntoView {
    let SourceAddSentencesParams { source_id } = utils::params()?;
    let source_id = source_id.expect("failed to get source_id");
    tracing::info!("Rendering SourceAddSentences {source_id}");

    let analyse_textarea_ref = NodeRef::<Textarea>::new();
    let analyse_act = Action::new(move |&()| {
        let textarea_val = analyse_textarea_ref
            .get()
            .expect("failed to get analyse_textarea_ref")
            .value();
        let client = get_client();
        async move { SendWrapper::new(client.segment_paragraph(source_id, &textarea_val)).await }
    });

    // source
    let source_res = utils::logged_in_resource!(get_source(source_id));
    let source_content = move |source: res::Source| {
        view! {
            <h2 class="subtitle">
                <A href=format!("/source/{source_id}")>{source.name}</A>
            </h2>
            <label class="label">
                "Paragraph"
                <textarea class="textarea" node_ref=analyse_textarea_ref/>
            </label>
            <button class="button is-primary mt-1" on:click=move |_ev| { analyse_act.dispatch(()); }>
                "Analyse"
            </button>
        }
    };
    let source_view = move |source: Option<res::Source>| match source {
        Some(source) => source_content(source).into_any(),
        None => view! { <div>"Loading source..."</div> }.into_any(),
    };

    // analysis
    let analysis_content = move |paragraph: res::SegmentedParagraph| {
        view! { <SegmentedParagraphView source_id=source_id paragraph /> }
    };
    let analysis_view = move |segmented: Option<res::SegmentedParagraph>| match segmented {
        Some(segments) => analysis_content(segments).into_any(),
        None => view! { <div>"Nothing analysed yet"</div> }.into_any(),
    };
    let analysis = move || {
        let view = if analyse_act.pending().get() {
            view! { <div>"Analysing..."</div> }.into_any()
        } else {
            let paragraph = analyse_act.value().get().transpose()?;
            analysis_view(paragraph).into_any()
        };
        WebResult::Ok(view)
    };
    let analysis = move || {
        view! {
            <ErrorBoundary fallback={utils::errors_fallback}>
                {analysis}
            </ErrorBoundary>
        }
    };

    let view = view! {
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
    source_id: Option<i32>,
    sentence_id: Option<i32>,
}
#[component]
pub fn SourceSentence() -> impl IntoView {
    let SourceSentenceParams {
        source_id,
        sentence_id,
    } = utils::params()?;
    let source_id = source_id.expect("failed to get source_id");
    let sentence_id = sentence_id.expect("failed to get sentence_id");
    tracing::info!("Rendering Sentence {source_id} {sentence_id}");

    let sentence_res = utils::logged_in_resource!(get_sentence(sentence_id));

    let reanalyse_act = Action::new(move |&()| {
        let client = get_client();
        async move { SendWrapper::new(client.segment_sentence(sentence_id)).await }
    });
    let delete_act = Action::new(move |&()| {
        let confirmed = leptos::prelude::window()
            .confirm_with_message("Are you sure you want to delete this sentence?")
            .map_err(WebError::from_js);
        let client = get_client();
        async move {
            let confirmed = confirmed?;
            let view = if confirmed {
                SendWrapper::new(client.delete_sentence(sentence_id)).await?;
                Some(view! { <Redirect path={format!("/source/{source_id}")} /> })
            } else {
                None
            };
            WebResult::Ok(view)
        }
    });

    // analysis
    let analysis_content = move |segmented_sentence: res::SegmentedSentence| {
        let on_successful_accept = Arc::new(move || {
            // todo sentence_res.refetch();
            reanalyse_act.value().set(None);
        });
        view! {
            <SegmentedSentenceView source_id sentence_id=Some(sentence_id) sentence={segmented_sentence.sentence} segments={segmented_sentence.segments} ignored_words={Arc::new(segmented_sentence.ignored_words)} on_successful_accept=on_successful_accept />
        }
    };
    let analysis_view =
        move |segmented: Option<res::SegmentedSentence>| segmented.map(analysis_content);
    let analysis = move || {
        let view = if reanalyse_act.pending().get() {
            view! { <div>"Analysing..."</div> }.into_any()
        } else {
            let segmented = reanalyse_act.value().get().transpose()?;
            analysis_view(segmented).into_any()
        };
        WebResult::Ok(view)
    };
    let analysis = move || {
        view! {
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
                    view! {
                        <li>
                            <div>{format!("{word} ({reading})")}</div>
                            <div>{translations}</div>
                        </li>
                    }
                } else {
                    view! {
                        <li>
                            <div>{word}</div>
                            <div>{translations}</div>
                        </li>
                    }
                }
            })
            .collect_view();
        view! {
            <div class="block">
                <div>{sentence.sentence}</div>
            </div>
            <div class="block">
                <button class="button is-primary" on:click=move |_ev| { reanalyse_act.dispatch(()); }>
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
                <button class="button is-danger" on:click=move |_ev| { delete_act.dispatch(()); }>
                    "Delete sentence"
                </button>
                <ActionView action=delete_act/>
            </div>
        }
    };
    let sentence_view = move |sentence: Option<res::SentenceDetails>| match sentence {
        Some(sentence) => sentence_content(sentence).into_any(),
        None => view! { <div>"Loading sentence..."</div> }.into_any(),
    };

    let view = view! {
        <LoginGuard require_login=true>
            <h2 class="subtitle">"Sentence"</h2>
            <ResourceView resource=sentence_res view=sentence_view/>
        </LoginGuard>
    };
    WebResult::Ok(view)
}

#[component]
pub fn DeckNew() -> impl IntoView {
    tracing::info!("Rendering DeckNew");

    let name_ref = NodeRef::<Input>::new();
    let save_act = Action::new(move |&()| {
        let name = name_ref.get().expect("failed to get name_ref").value();
        let client = get_client();
        async move {
            if name.is_empty() {
                return Err(WebError {
                    message: "Deck name cannot be empty!".to_string(),
                });
            }
            let id = SendWrapper::new(client.new_deck(&name)).await?;
            WebResult::Ok(view! { <Redirect path=format!("/deck/{id}") /> })
        }
    });

    view! {
        <LoginGuard require_login=true>
            <h2 class="subtitle">"Adding new deck"</h2>
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
                        "Create deck"
                    </button>
                    <ActionView action=save_act/>
                </div>
            </form>
        </LoginGuard>
    }
}

#[derive(Debug, Clone, PartialEq, Params)]
pub struct DeckParams {
    deck_id: Option<i32>,
}
#[component]
pub fn Deck() -> impl IntoView {
    let DeckParams { deck_id } = utils::params()?;
    let deck_id = deck_id.expect("failed to get deck_id");
    tracing::info!("Rendering Deck {deck_id}");

    // resources
    let deck_res = utils::logged_in_resource!(get_deck(deck_id));
    let sources_res = utils::logged_in_resource!(get_sources());

    #[derive(Clone, Copy)]
    struct SourceRefs {
        source_id: i32,
        include_words: NodeRef<Input>,
        include_kanji: NodeRef<Input>,
        word_threshold: NodeRef<Input>,
        kanji_threshold: NodeRef<Input>,
    }

    // actions
    let name_ref = NodeRef::<Input>::new();
    let (source_refs, set_source_checkbox_refs) = leptos::prelude::signal(Vec::<SourceRefs>::new());
    let (update_result_message, set_update_result_message) =
        leptos::prelude::signal((None::<&'static str>, None::<TimeoutHandle>));
    let update_act = Action::new(move |&()| {
        let client = get_client();
        let name = name_ref.get().expect("failed to get name_ref").value();
        let mut included_sources = Vec::new();

        async move {
            for SourceRefs {
                source_id,
                include_words,
                include_kanji,
                word_threshold,
                kanji_threshold,
            } in source_refs.get()
            {
                if include_words
                    .get()
                    .expect("failed to get include_words")
                    .checked()
                {
                    let threshold = word_threshold
                        .get()
                        .expect("failed to get word_threshold")
                        .value()
                        .parse()
                        .map_err(|e| {
                            WebError::new(format!("Failed to parse threshold as number: {e}"))
                        })?;
                    if threshold < 1 {
                        return Err(WebError::new("Threshold cannot be lower than 1"));
                    }
                    included_sources.push(req::IncludedSource {
                        source_id,
                        threshold,
                        kind: req::IncludedSourceKind::Word,
                    });
                }
                if include_kanji
                    .get()
                    .expect("failed to get include_kanji")
                    .checked()
                {
                    let threshold = kanji_threshold
                        .get()
                        .expect("failed to get kanji_threshold")
                        .value()
                        .parse()
                        .map_err(|e| {
                            WebError::new(format!("Failed to parse threshold as number: {e}"))
                        })?;
                    if threshold < 1 {
                        return Err(WebError::new("Threshold cannot be lower than 1"));
                    }
                    included_sources.push(req::IncludedSource {
                        source_id,
                        threshold,
                        kind: req::IncludedSourceKind::Kanji,
                    });
                }
            }

            SendWrapper::new(client.update_deck(deck_id, &name, &included_sources)).await?;
            // todo deck_res.refetch();
            if let Some(handle) = update_result_message.get().1 {
                handle.clear();
            }
            let handle = leptos::prelude::set_timeout_with_handle(
                move || {
                    set_update_result_message.set((None, None));
                },
                Duration::from_secs(4),
            )
            .ok();
            set_update_result_message.set((Some("Updated deck!"), handle));

            WebResult::Ok(())
        }
    });
    let delete_act = Action::new(move |&()| {
        let confirmed = leptos::prelude::window()
            .confirm_with_message("Are you sure you want to delete this deck?")
            .map_err(WebError::from_js);
        let client = get_client();
        async move {
            let confirmed = confirmed?;
            let view = if confirmed {
                SendWrapper::new(client.delete_deck(deck_id)).await?;
                Some(view! { <Redirect path="/" /> })
            } else {
                None
            };
            WebResult::Ok(view)
        }
    });

    // views
    let deck_sources_content = move |deck: res::DeckDetails, sources: Vec<res::Source>| {
        let client = get_client();
        let today = chrono::Utc::now().date_naive();
        let filename = format!("lbr-{}-{}.apkg", deck.name, today);
        let generate_deck_url = client.generate_deck_url(deck_id, &filename);

        let mut refs = vec![];
        let sources_list = sources
            .into_iter()
            .map(|s| {
                let include_words = NodeRef::<Input>::new();
                let word_threshold = NodeRef::<Input>::new();
                let (words_checked, word_threshold_val) =
                    deck
                        .sources
                        .iter()
                        .find(|ds| ds.id == s.id)
                        .filter(|ds| matches!(ds.kind, res::DeckSourceKind::Word))
                        .map(|ds| (true, ds.threshold))
                        .unwrap_or((false, 1));
                let (kanji_checked, kanji_threshold_val) =
                    deck
                        .sources
                        .iter()
                        .find(|ds| ds.id == s.id)
                        .filter(|ds| matches!(ds.kind, res::DeckSourceKind::Kanji))
                        .map(|ds| (true, ds.threshold))
                        .unwrap_or((false, 1));
                let include_kanji = NodeRef::<Input>::new();
                let kanji_threshold = NodeRef::<Input>::new();

                refs.push(SourceRefs {
                    source_id: s.id,
                    include_words,
                    include_kanji,
                    word_threshold,
                    kanji_threshold
                });
                view! {
                    <li>
                        {s.name}
                        <br/>
                        <label class="checkbox">
                            <input class="checkbox mr-1" type="checkbox" checked=words_checked node_ref=include_words/>
                            "Words"
                        </label>
                        <br/>
                        <label>
                            "Minimum sentence count:"
                            <input class="input ml-1" style="max-width: 16rem;" type="number" min=1 max=i32::MAX value=word_threshold_val node_ref=word_threshold/>
                        </label>
                        <br/>
                        <label class="checkbox">
                            <input class="checkbox mr-1" type="checkbox" checked=kanji_checked node_ref=include_kanji/>
                            "Kanji"
                        </label>
                        <br/>
                        <label>
                            "Minimum word count:"
                            <input class="input ml-1" style="max-width: 16rem;" type="number" min=1 max=i32::MAX value=kanji_threshold_val node_ref=kanji_threshold/>
                        </label>
                    </li>
                }
            })
            .collect_view();
        set_source_checkbox_refs.set(refs);

        view! {
            <h2 class="subtitle">{format!("Viewing deck {}", deck.name)}</h2>
            <div class="block">
                <a href=generate_deck_url download=filename class="button is-primary">
                    "Generate deck"
                </a>
            </div>
            <div class="block">
                <h3 class="subtitle">"Edit deck"</h3>
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
                        "Update deck"
                    </button>
                    <ActionView action=update_act/>
                    {move || update_result_message.get().0}
                </form>
            </div>
            <div class="block">
                <button class="button is-danger" on:click=move |_ev| { delete_act.dispatch(()); }>
                    "Delete deck"
                </button>
                <ActionView action=delete_act/>
            </div>
        }
        .into_view()
    };
    let deck_sources_view = move |deck: Option<res::DeckDetails>,
                                  sources: Option<Vec<res::Source>>| {
        match (deck, sources) {
            (Some(d), Some(s)) => deck_sources_content(d, s).into_any(),
            (None, _) => view! { <div>"Loading deck..."</div> }.into_any(),
            (_, None) => view! { <div>"Loading sources..."</div> }.into_any(),
        }
    };
    let deck_sources = move || match (deck_res.get(), sources_res.get()) {
        (Some(Ok(Some(deck_res))), Some(Ok(Some(sources_res)))) => Ok(Some(
            deck_sources_view(Some(deck_res), Some(sources_res)).into_view(),
        )),
        (Some(Ok(Some(deck_res))), None) => {
            Ok(Some(deck_sources_view(Some(deck_res), None).into_view()))
        }
        (None, Some(Ok(Some(sources_res)))) => {
            Ok(Some(deck_sources_view(None, Some(sources_res)).into_view()))
        }
        (Some(Ok(None)), _) | (_, Some(Ok(None))) => Ok(None),
        (Some(Err(err)), _) | (_, Some(Err(err))) => Err(err),
        (None, None) => Ok(Some(deck_sources_view(None, None).into_view())),
    };

    let view = view! {
        <LoginGuard require_login=true>
            //<Suspense fallback={move || deck_sources_view(None, None)}>
                <ErrorBoundary fallback={utils::errors_fallback}>
                    {deck_sources}
                </ErrorBoundary>
            //</Suspense>
        </LoginGuard>
    };
    WebResult::Ok(view)
}

#[component]
pub fn IgnoredWords() -> impl IntoView {
    tracing::info!("Rendering IgnoredWords");

    let delete_act = Action::new(move |word_id: &i32| {
        let confirmed = leptos::prelude::window()
            .confirm_with_message("Are you sure you want to delete this ignored word?")
            .map_err(|e| WebError::new(format!("{e:?}")));
        let word_id = *word_id;
        let client = get_client();
        async move {
            if confirmed? {
                SendWrapper::new(client.delete_ignored_word(word_id)).await?;
            }
            Ok(())
        }
    });

    let ignored_words_res = utils::logged_in_resource!(get_ignored_words());
    let ignored_words_content = move |mut ignored_words: Vec<res::IgnoredWord>| {
        if ignored_words.is_empty() {
            return view! { <div>"No ignored words"</div> }.into_any();
        }
        ignored_words.sort_unstable_by_key(|iw| iw.word_id);
        let ignored_words = ignored_words
            .into_iter()
            .map(|iw| {
                let translations = iw.translations.join(", ");
                let written_forms = iw
                    .written_forms
                    .into_iter()
                    .map(|wf| {
                        let readings = wf.readings.join(", ");
                        if readings.is_empty() {
                            wf.written_form
                        } else {
                            format!("{} ({})", wf.written_form, readings)
                        }
                    })
                    .collect::<Vec<_>>().join(", ");
                let word_id = format!("[{}]", iw.word_id);
                view! {
                    <div class="column">
                        <div class="box">
                            <div>
                                <span class="has-text-weight-bold">
                                    {word_id}
                                </span>
                            </div>
                            <div>
                                <div>
                                    <span class="has-text-weight-bold">"Written forms"</span>
                                    ": "
                                    {written_forms}
                                </div>
                            </div>
                            <div>
                                <span class="has-text-weight-bold">"Translations"</span>
                                ": "
                                {translations}
                            </div>
                            <button class="button is-danger mt-2" on:click=move |_ev| { delete_act.dispatch(iw.word_id); }>"Delete ignored word"</button>
                        </div>
                    </div>
                }
            })
            .collect_view();
        view! {
            <div class="columns is-flex-wrap-wrap">
                {ignored_words}
            </div>
        }
        .into_any()
    };
    let ignored_words_view = move |ignored_words: Option<Vec<res::IgnoredWord>>| match ignored_words
    {
        Some(ignored_words) => ignored_words_content(ignored_words).into_any(),
        None => view! { <div>"Loading..."</div> }.into_any(),
    };

    view! {
        <LoginGuard require_login=true>
            <h2 class="subtitle">"Ignored words"</h2>
            <ActionView action=delete_act/>
            <ResourceView resource=ignored_words_res view=ignored_words_view/>
        </LoginGuard>
    }
}

#[component]
pub fn Login() -> impl IntoView {
    tracing::info!("Rendering Login");

    let redirect = move || {
        leptos_router::hooks::use_query_map()
            .get()
            .get("redirect")
            .unwrap_or_else(|| "/".to_string())
    };

    // form
    let email_ref = NodeRef::<Input>::new();
    let password_ref = NodeRef::<Input>::new();
    let submission_act = Action::new(move |&()| {
        tracing::info!("Logging in");
        let email = email_ref.get().expect("failed to get email_ref").value();
        let password = password_ref
            .get()
            .expect("failed to get password_ref")
            .value();
        let client = get_client();
        async move {
            if email.is_empty() {
                return Err(WebError::new("Email cannot be empty"));
            }
            if password.is_empty() {
                return Err(WebError::new("Password cannot be empty"));
            }
            SendWrapper::new(client.login(email.as_str(), password.as_str())).await?;
            let view = move || view! { <Redirect path=redirect() /> };
            WebResult::Ok(view)
        }
    });

    let password_visible = RwSignal::new(false);
    let password_visibility_toggle = move || {
        if password_visible.get() {
            view! { <button class="button" on:click=move |_ev| password_visible.set(false)>"Hide password"</button> }.into_any()
        } else {
            view! { <button class="button" on:click=move |_ev| password_visible.set(true)>"Show password"</button> }.into_any()
        }
    };
    let password_input_type = move || {
        if password_visible.get() {
            "text"
        } else {
            "password"
        }
    };

    Effect::new(move |_| {
        if let Some(email_ref) = email_ref.get() {
            email_ref.focus().expect("failed to get email_ref");
        }
    });

    view! {
        <LoginGuard require_login=false>
            <h2 class="subtitle">"Login"</h2>
            <form>
                <label class="label">
                    "Email"
                    <input class="input" node_ref=email_ref/>
                </label>
                <label class="label">
                    "Password"
                    <input class="input" type=password_input_type node_ref=password_ref/>
                </label>
                <button class="button mr-2" type="submit" on:click={move |ev| {
                    ev.prevent_default();
                    submission_act.dispatch(());
                }}>
                    "Login"
                </button>
                {password_visibility_toggle}
            </form>
            <ActionView action=submission_act/>
        </LoginGuard>
    }
}

#[component]
pub fn Register() -> impl IntoView {
    tracing::info!("Rendering Register");

    // form
    let email_ref = NodeRef::<Input>::new();
    let password_ref = NodeRef::<Input>::new();
    let repeat_password_ref = NodeRef::<Input>::new();
    let submit = Action::new(move |&()| {
        tracing::info!("Registering");
        let email = email_ref.get().expect("failed to get email_ref").value();
        let password = password_ref
            .get()
            .expect("failed to get password_ref")
            .value();
        let repeat_password = repeat_password_ref
            .get()
            .expect("failed to get repeat_password_ref")
            .value();
        let client = get_client();
        async move {
            if email.is_empty() {
                return Err(WebError::new("Email cannot be empty"));
            }
            if password.is_empty() {
                return Err(WebError::new("Password cannot be empty"));
            }
            if password != repeat_password {
                return Err(WebError::new("Passwords don't match"));
            }
            SendWrapper::new(client.register(&email, &password)).await?;
            WebResult::Ok(view! { <Redirect path="/login" /> })
        }
    });

    let password_visible: RwSignal<bool> = RwSignal::new(false);
    let password_visibility_toggle = move || {
        if password_visible.get() {
            view! { <button class="button" on:click=move |_ev| password_visible.set(false)>"Hide passwords"</button> }.into_any()
        } else {
            view! { <button class="button" on:click=move |_ev| password_visible.set(true)>"Show passwords"</button> }.into_any()
        }
    };
    let password_input_type = move || {
        if password_visible.get() {
            "text"
        } else {
            "password"
        }
    };

    Effect::new(move |_| {
        if let Some(email_ref) = email_ref.get() {
            email_ref.focus().expect("failed to get email_ref");
        }
    });

    view! {
        <LoginGuard require_login=false>
            <h2 class="subtitle">"Register"</h2>
            <form>
                <label class="label">
                    "Email"
                    <input class="input" node_ref=email_ref/>
                </label>
                <label class="label">
                    "Password"
                    <input class="input" type=password_input_type node_ref=password_ref/>
                </label>
                <label class="label">
                    "Repeat password"
                    <input class="input" type=password_input_type node_ref=repeat_password_ref/>
                </label>
                <button class="button mr-2" type="submit" on:click={move |ev| {
                    ev.prevent_default();
                    submit.dispatch(());
                }}>
                    "Register"
                </button>
                {password_visibility_toggle}
            </form>
            <ActionView action=submit/>
        </LoginGuard>
    }
}
