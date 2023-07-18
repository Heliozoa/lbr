//! Components related to sentence analysis.

use crate::{context::get_client, error::WebResult};
use lbr_api::{request as req, response as res};
use leptos::*;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[component]
pub fn SegmentedParagraphView(
    cx: Scope,
    source_id: i32,
    segmented: Vec<res::SegmentedSentence>,
) -> impl IntoView {
    let active_sentence = leptos::create_rw_signal(cx, 0usize);
    let completed_sentences = leptos::create_rw_signal(cx, HashSet::<usize>::new());
    let is_active = move |idx: usize| active_sentence.get() == idx;
    let is_complete = move |idx: usize| completed_sentences.get().contains(&idx);
    let sentence_selection = segmented
        .iter()
        .enumerate()
        .map(|(idx, segmented_sentence)| {
            let text = move || {
                if is_complete(idx) {
                    "Complete"
                } else if is_active(idx) {
                    "Active"
                } else {
                    "Select"
                }
            };
            let class = move || {
                if is_complete(idx) {
                    "box has-background-info-light"
                } else if is_active(idx) {
                    "box has-background-success-light"
                } else {
                    "box has-background-warning-light"
                }
            };
            let button_class = move || {
                if is_complete(idx) {
                    "button mt-2"
                } else if is_active(idx) {
                    "button mt-2 is-light"
                } else {
                    "button mt-2 is-primary"
                }
            };
            let is_disabled = move || {
                is_complete(idx) || is_active(idx)
            };
            view! { cx,
                <div class=class>
                    <div>
                        {segmented_sentence.sentence.clone()}
                    </div>
                    <button class=button_class disabled=is_disabled on:click=move |_ev| active_sentence.set(idx)>
                        {text}
                    </button>
                </div>
            }
        })
        .collect_view(cx);
    let segmented_sentences = segmented
        .into_iter()
        .enumerate()
        .map(|(idx, segmented_sentence)| {
            let class = move || {
                if is_active(idx) {
                    ""
                } else {
                    "is-hidden"
                }
            };
            view! { cx,
                <div class=class>
                    <SegmentedSentenceView source_id sentence_id=None segmented_sentence on_successful_accept=Arc::new(move || {
                        completed_sentences.update(|cs| {
                            cs.insert(idx);
                        });
                        active_sentence.update(|acs| {
                            *acs += 1;
                        });
                        let _ = leptos::window().location().set_hash("paragraph-segmentation");
                    }) />
                </div>
            }
        })
        .collect_view(cx);
    view! { cx,
        <div class="block">
            <div id="paragraph-segmentation" class="subtitle">"Paragraph segmentation"</div>
            {sentence_selection}
            {segmented_sentences}
        </div>
    }
}

#[derive(Debug, Clone)]
pub struct Component {
    word_id: i32,
    idx_start: usize,
    idx_end: usize,
    status: Status,
    reading: Option<String>,
    reading_override: String,
}

impl Component {
    fn ready(&self) -> bool {
        !matches!(self.status, Status::Undecided)
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Accept,
    Decline,
    Ignore,
    Undecided,
}

type WordIdToComponents = HashMap<i32, Vec<(ReadSignal<Component>, WriteSignal<Component>)>>;
type PhraseToComponents =
    HashMap<usize, Vec<(usize, (ReadSignal<Component>, WriteSignal<Component>))>>;
type SeqsToComponent =
    HashMap<(usize, usize, usize), (ReadSignal<Component>, WriteSignal<Component>)>;

#[derive(Debug, Clone)]
struct Form {
    word_id_to_components: Arc<WordIdToComponents>,
    phrase_to_components: Arc<PhraseToComponents>,
    seqs_to_component: Arc<SeqsToComponent>,
}

impl Form {
    fn init(cx: Scope, segmented_sentence: &res::SegmentedSentence) -> Self {
        let mut word_id_to_components: WordIdToComponents = HashMap::new();
        let mut phrase_to_components: PhraseToComponents = HashMap::new();
        let mut seqs_to_component: SeqsToComponent = HashMap::new();

        let mut phrase_seq = 0;
        let mut phrase_idx = 0;
        for segment in &segmented_sentence.segments {
            let mut phrase_components = Vec::new();
            match segment {
                res::Segment::Phrase {
                    phrase,
                    interpretations,
                } => {
                    phrase_idx += segmented_sentence.sentence[phrase_idx..]
                        .find(phrase.as_str())
                        .unwrap();
                    let mut interpretation_seq = 0;
                    for interpretation in interpretations {
                        let mut interpretation_idx = phrase_idx;
                        let mut interpretation_components = Vec::new();
                        let mut component_seq = 0;
                        for component in &interpretation.components {
                            interpretation_idx += segmented_sentence.sentence[interpretation_idx..]
                                .find(&component.word)
                                .unwrap();
                            let idx_end = interpretation_idx + component.word.len();
                            if let Some(word_id) = component.word_id {
                                let status: Status = if interpretations.len() == 1 {
                                    // pre-emptively accept "clear" cases
                                    Status::Accept
                                } else if segmented_sentence.ignored_words.contains(&word_id) {
                                    Status::Ignore
                                } else {
                                    Status::Undecided
                                };
                                let reading = if component.word == component.reading_hiragana {
                                    None
                                } else {
                                    Some(component.reading_hiragana.clone())
                                };
                                let signal = leptos::create_signal(
                                    cx,
                                    Component {
                                        idx_start: interpretation_idx,
                                        idx_end,
                                        word_id,
                                        status,
                                        reading,
                                        reading_override: String::new(),
                                    },
                                );
                                word_id_to_components
                                    .entry(word_id)
                                    .or_default()
                                    .push(signal);
                                seqs_to_component.insert(
                                    (phrase_seq, interpretation_seq, component_seq),
                                    signal,
                                );
                                interpretation_components.push(signal);
                                phrase_components.push((interpretation_seq, signal));
                            }
                            component_seq += 1;
                        }
                        interpretation_seq += 1;
                    }
                }
                res::Segment::Other(other) => {
                    phrase_idx += other.len();
                }
            }
            if !phrase_components.is_empty() {
                phrase_to_components.insert(phrase_seq, phrase_components);
            }
            phrase_seq += 1;
        }

        Self {
            word_id_to_components: Arc::new(word_id_to_components),
            phrase_to_components: Arc::new(phrase_to_components),
            seqs_to_component: Arc::new(seqs_to_component),
        }
    }
}

#[component]
pub fn SegmentedSentenceView(
    cx: Scope,
    source_id: i32,
    sentence_id: Option<i32>,
    segmented_sentence: res::SegmentedSentence,
    on_successful_accept: Arc<dyn Fn() -> ()>,
) -> impl IntoView {
    let form = Form::init(cx, &segmented_sentence);

    // accept is enabled if no word forms are left undecided
    let components = form
        .clone()
        .seqs_to_component
        .values()
        .map(|v| v.0)
        .collect::<Vec<_>>();
    let accept_disabled = move || components.iter().any(|read| !read().ready());
    let submit = leptos::create_action(cx, move |sentence: &req::SegmentedSentence| {
        let client = get_client(cx);
        let sentence = sentence.clone();
        let on_successful_accept = on_successful_accept.clone();
        async move {
            if let Some(sentence_id) = sentence_id {
                client.update_sentence(sentence_id, &sentence).await?
            } else {
                client.new_sentence(source_id, &sentence).await?
            }
            on_successful_accept();
            WebResult::Ok(())
        }
    });

    let components = form
        .clone()
        .seqs_to_component
        .values()
        .map(|v| v.0)
        .collect::<Vec<_>>();
    let accept_sentence = move |_ev| {
        let mut words = Vec::new();
        let mut ignore_words = HashSet::new();
        for component in components.iter().map(|read| read()) {
            match component.status {
                Status::Accept => {
                    let reading = if component.reading_override.trim().is_empty() {
                        component.reading
                    } else {
                        Some(component.reading_override)
                    };
                    words.push(req::Word {
                        id: component.word_id,
                        reading,
                        idx_start: i32::try_from(component.idx_start).unwrap_or_default(),
                        idx_end: i32::try_from(component.idx_end).unwrap_or_default(),
                    })
                }
                Status::Ignore => {
                    ignore_words.insert(component.word_id);
                }
                Status::Decline => {}
                Status::Undecided => panic!("Cannot accept sentence with undecided components"),
            }
        }
        let segmented_sentence = req::SegmentedSentence {
            sentence: segmented_sentence.sentence.clone(),
            words,
            ignore_words,
        };
        submit.dispatch(segmented_sentence);
    };

    // show each segment with interpretations
    let mut phrase_seq = 0;
    let ignored_words = Arc::new(segmented_sentence.ignored_words);
    let mut unknown_or_ignored_storage = String::new();
    let sentence_segments = segmented_sentence.segments.into_iter().map(|s| {
        let segment_view = match s {
            res::Segment::Phrase {
                phrase,
                interpretations,
            } => {
                let all_unknown_or_ignored = interpretations
                    .iter()
                    .flat_map(|i| &i.components)
                    .all(|c| match c.word_id {
                        Some(word_id) => ignored_words.contains(&word_id),
                        None => true,
                    });
                if all_unknown_or_ignored {
                    unknown_or_ignored_storage += &phrase;
                    None
                } else {
                    let preceding_unknown_or_ignored_words = if !unknown_or_ignored_storage
                        .is_empty()
                    {
                        let ret = Some(view! { cx,
                            <div class="box has-background-info-light">
                                <div class="has-text-weight-bold">{unknown_or_ignored_storage.clone()}</div>
                            </div>
                        });
                        unknown_or_ignored_storage.clear();
                        ret
                    } else {
                        None
                    };
                    Some(
                        view! { cx,
                            {preceding_unknown_or_ignored_words}
                            <PhraseView
                                phrase
                                interpretations
                                form=form.clone()
                                phrase_seq
                                ignored_words=ignored_words.clone()
                            />
                        }
                        .into_view(cx),
                    )
                }
            }
            res::Segment::Other(other) => {
                unknown_or_ignored_storage += &other;
                None
            },
        };
        phrase_seq += 1;
        segment_view
    }).flatten().collect_view(cx);
    let tailing_unknown_or_ignored_words = if !unknown_or_ignored_storage.is_empty() {
        Some(view! { cx,
            <div class="box has-background-info-light">
                <div class="has-text-weight-bold">{unknown_or_ignored_storage}</div>
            </div>
        })
    } else {
        None
    };

    view! { cx,
        <div class="block">
            <div class="subtitle">"Sentence segmentation"</div>
            {sentence_segments}
            {tailing_unknown_or_ignored_words}
            <button class="button is-primary" disabled=accept_disabled on:click=accept_sentence>"Accept sentence"</button>
        </div>
    }
}

#[component]
fn PhraseView(
    cx: Scope,
    phrase: String,
    interpretations: Vec<res::Interpretation>,
    form: Form,
    phrase_seq: usize,
    ignored_words: Arc<HashSet<i32>>,
) -> impl IntoView {
    let mut interpretation_seq = 0;
    let interpretations = interpretations
        .into_iter()
        .filter_map(|interpretation| {
            // filter out interpretations where all components are ignored
            let all_unknown_or_ignored =
                interpretation.components.iter().all(|c| match c.word_id {
                    Some(word_id) => ignored_words.contains(&word_id),
                    None => true,
                });
            if all_unknown_or_ignored {
                None
            } else {
                let view = view! { cx,
                    <InterpretationView
                        interpretation
                        form=form.clone()
                        phrase_seq
                        interpretation_seq
                        ignored_words=ignored_words.clone()
                    />
                };
                interpretation_seq += 1;
                Some(view)
            }
        })
        .collect_view(cx);
    let form_clone = form.clone();
    let any_undecided = move || {
        let any_undecided = form_clone
            .phrase_to_components
            .get(&phrase_seq)
            .unwrap()
            .iter()
            .any(|(_seq, (read, _write))| matches!(read().status, Status::Undecided));
        any_undecided
    };
    let form_clone = form.clone();
    let any_accepted = move || {
        let any_accepted = form_clone
            .phrase_to_components
            .get(&phrase_seq)
            .unwrap()
            .iter()
            .any(|(_seq, (read, _write))| matches!(read().status, Status::Accept));
        any_accepted
    };
    let form_clone = form.clone();
    let any_skipped = move || {
        let any_skipped = form_clone
            .phrase_to_components
            .get(&phrase_seq)
            .unwrap()
            .iter()
            .any(|(_seq, (read, _write))| matches!(read().status, Status::Decline));
        any_skipped
    };
    let form_clone = form.clone();
    let all_skipped = move || {
        let all_skipped = form_clone
            .phrase_to_components
            .get(&phrase_seq)
            .unwrap()
            .iter()
            .all(|(_seq, (read, _write))| matches!(read().status, Status::Decline));
        all_skipped
    };

    let skipped_clone = any_skipped.clone();
    let box_class = move || {
        if any_undecided() {
            "box has-background-danger-light"
        } else if any_accepted() {
            "box has-background-success-light"
        } else if skipped_clone() {
            "box has-background-warning-light"
        } else {
            "box has-background-info-light"
        }
    };
    let form_clone = form.clone();
    let skip_phrase = move |_ev| {
        form_clone
            .phrase_to_components
            .get(&phrase_seq)
            .unwrap()
            .iter()
            .for_each(|(_seq, (_read, write))| {
                write.update(|f| {
                    f.status = Status::Decline;
                })
            });
    };
    view! { cx,
        <div class=box_class>
            <div class="has-text-weight-bold">{phrase}</div>
            <hr/>
            <div class="columns is-flex is-flex-direction-row is-flex-wrap-wrap">
                {interpretations}
            </div>
            <button class="button" disabled=all_skipped on:click=skip_phrase>"Skip phrase"</button>
        </div>
    }
}

#[component]
fn InterpretationView(
    cx: Scope,
    interpretation: res::Interpretation,
    form: Form,
    phrase_seq: usize,
    interpretation_seq: usize,
    ignored_words: Arc<HashSet<i32>>,
) -> impl IntoView {
    let mut component_seq = 0;
    let components = interpretation
        .components
        .into_iter()
        .map(|component| {
            let unknown_or_ignored = match component.word_id {
                Some(word_id) => ignored_words.contains(&word_id),
                None => true,
            };
            if unknown_or_ignored {
                view! { cx,
                    <div>{format!("{} (ignored)", component.word)}</div>
                }
                .into_view(cx)
            } else {
                let view = view! { cx,
                    <ComponentView
                        show_reading={interpretation.reading_hiragana != component.reading_hiragana}
                        component
                        form=form.clone()
                        phrase_seq
                        interpretation_seq
                        component_seq
                    />
                };
                component_seq += 1;
                view.into_view(cx)
            }
        })
        .collect_view(cx);
    view! { cx,
        <div class="column is-flex is-flex-direction-column">
            <div>{format!("Score: {}", interpretation.score)}</div>
            <div>{format!("Reading: {}", interpretation.reading_hiragana)}</div>
            {components}
        </div>
    }
}

#[component]
fn ComponentView(
    cx: Scope,
    show_reading: bool,
    component: res::WordInfo,
    form: Form,
    phrase_seq: usize,
    interpretation_seq: usize,
    component_seq: usize,
) -> impl IntoView {
    let meanings = component
        .meanings
        .into_iter()
        .map(|meaning| view! { cx, <MeaningView meaning/> })
        .collect_view(cx);
    let reading_view = show_reading.then(|| {
        view! { cx, <span>{&component.reading_hiragana}</span>
        <br/> }
    });
    let (read, write) = form
        .seqs_to_component
        .get(&(phrase_seq, interpretation_seq, component_seq))
        .unwrap()
        .clone();
    let accept = move |_ev| {
        // unaccept components of other interpretations
        form.phrase_to_components
            .get(&phrase_seq)
            .unwrap()
            .iter()
            .for_each(|(seq, (_read, write))| {
                if *seq != interpretation_seq {
                    write.update(|c| {
                        if !matches!(c.status, Status::Ignore) {
                            c.status = Status::Decline;
                        }
                    });
                }
            });
        write.update(|c| c.status = Status::Accept);
    };
    let decline = move |_ev| {
        write.update(|c| c.status = Status::Decline);
    };
    let ignore = move |_ev| {
        // ignore all words with the same id
        if let Some(word_id) = component.word_id {
            form.word_id_to_components
                .get(&word_id)
                .unwrap()
                .iter()
                .for_each(|(_read, write)| write.update(|c| c.status = Status::Ignore));
        }
        write.update(|c| c.status = Status::Ignore);
    };
    let accepted = move || matches!(read().status, Status::Accept { .. });
    let declined = move || matches!(read().status, Status::Decline);
    let ignored = move || matches!(read().status, Status::Ignore);
    let (reading_override, set_reading_override) = leptos::create_signal(cx, String::new());
    view! { cx,
        <div>
            {reading_view}
            <label class="label">"Override reading"
                <input class="input" prop:value=reading_override on:input=move |ev| {
                    let new_value = leptos::event_target_value(&ev);
                    set_reading_override.update(|reading_override| {
                        *reading_override = new_value.clone()
                    });
                    form.seqs_to_component.get(&(phrase_seq, interpretation_seq, component_seq)).unwrap().1.update(|c| {
                        c.reading_override = new_value;
                    });
                }/>
            </label>
            <br/>
        </div>
        <div>"Meanings:"</div>
        <div class="content">
            <ul>
                {meanings}
            </ul>
        </div>
        <div class="columns is-flex-wrap-wrap is-centered">
            <div class="column">
                <button class="button" style="width: 100%" disabled=accepted on:click=accept>"Accept"</button>
            </div>
            <div class="column">
                <button class="button" style="width: 100%" disabled=declined on:click=decline>"Decline"</button>
            </div>
            <div class="column">
                <button class="button" style="width: 100%" disabled=ignored on:click=ignore>"Ignore"</button>
            </div>
        </div>
    }
}

#[component]
pub fn MeaningView(cx: Scope, meaning: res::Meaning) -> impl IntoView {
    let contents = match meaning.meaning_info {
        Some(info) => format!("{} ({})", meaning.meaning, info),
        None => meaning.meaning,
    };
    view! { cx, <li>{contents}</li> }
}
