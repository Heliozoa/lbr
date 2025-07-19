//! Components related to sentence analysis.

use crate::{context::get_client, error::WebResult};
use itertools::Itertools;
use lbr_api::{request as req, response as res};
use leptos::prelude::*;
use send_wrapper::SendWrapper;
use std::{cell::RefCell, cmp::Ordering, collections::HashSet, ops::Range, rc::Rc, sync::Arc};

#[component]
pub fn SegmentedParagraphView(source_id: i32, paragraph: res::SegmentedParagraph) -> impl IntoView {
    let active_sentence = RwSignal::new(0usize);
    let completed_sentences = RwSignal::new(HashSet::<usize>::new());
    let sentence_is_active = move |idx: usize| active_sentence.read() == idx;
    let sentences_view = paragraph
        .sentences
        .iter()
        .enumerate()
        .map(|(idx, s)| {
            let sentence_button_class = move || {
                if sentence_is_active(idx) {
                    "button mt-2 is-light"
                } else {
                    "button mt-2 is-primary"
                }
            };
            let snippet_end = s.sentence.chars().take(4).map(|c| c.len_utf8()).sum();
            view! {
                <div>
                    <button class=sentence_button_class on:click=move |_ev| active_sentence.set(idx)>
                        {idx}: {s.sentence[..snippet_end].to_string()}
                    </button>
                </div>
            }
        })
        .collect_view();

    let ignored_words = Arc::new(paragraph.ignored_words);
    let segmented_sentence_views = paragraph
        .sentences
        .into_iter()
        .enumerate()
        .map(move |(idx, segmented_sentence)| {
            let class = move || {
                if sentence_is_active(idx) {
                    ""
                } else {
                    "is-hidden"
                }
            };
            let on_successful_accept = Arc::new(move || {
                completed_sentences.update(|cs| {
                    cs.insert(idx);
                });
                active_sentence.update(|acs| {
                    *acs += 1;
                });
                let _ = leptos::prelude::window()
                    .location()
                    .set_hash("paragraph-segmentation");
            });
            view! {
                <div class=class>
                    <SegmentedSentenceView
                        source_id={source_id}
                        sentence_id={None}
                        sentence={segmented_sentence.sentence}
                        segments={segmented_sentence.segments}
                        ignored_words={ignored_words.clone()}
                        on_successful_accept={on_successful_accept}
                    />
                </div>
            }
        })
        .collect_view();

    view! {
        {sentences_view}
        {segmented_sentence_views}
    }
}

#[derive(Debug, Clone)]
struct FormWord {
    field_id: i32,
    word_id: i32,
    range: Range<usize>,
    text_word: String,
    text_reading: Option<String>,
    db_word: String,
    db_reading: Option<String>,
    score: i32,
    meanings: Vec<String>,
    tail: Option<String>,
}

#[derive(Debug)]
enum FormWordOr {
    FormWord(FormWord),
    Ignored {
        text_word: String,
        range: Range<usize>,
    },
    Unknown {
        text_word: String,
        range: Range<usize>,
    },
}

impl FormWordOr {
    fn range(&self) -> Range<usize> {
        match self {
            FormWordOr::FormWord(fw) => fw.range.clone(),
            FormWordOr::Ignored { range, .. } => range.clone(),
            FormWordOr::Unknown { range, .. } => range.clone(),
        }
    }
}

#[component]
pub fn SegmentedSentenceView(
    source_id: i32,
    sentence_id: Option<i32>,
    sentence: String,
    segments: Vec<res::ApiSegment>,
    ignored_words: Arc<HashSet<i32>>,
    on_successful_accept: Arc<dyn Fn() + Send + Sync>,
) -> impl IntoView {
    // convert the words into a more convenient form
    let field_id = Rc::new(RefCell::new(0));
    let form_words = segments
        .into_iter()
        .flat_map(|s| {
            let s_range = s.range.clone();
            let sentence = &sentence;
            let ignored_words = &ignored_words;
            let field_id = field_id.clone();
            s.interpretations.clone().into_iter().map(move |i| {
                let text_word = sentence[s_range.clone()].to_string();

                let Some(word_id) = i.word_id else {
                    return FormWordOr::Unknown {
                        text_word,
                        range: s_range.clone(),
                    };
                };
                // check ignored
                if ignored_words.contains(&word_id) {
                    return FormWordOr::Ignored {
                        text_word,
                        range: s_range.clone(),
                    };
                }

                // format meanings
                let meanings = i
                    .meanings
                    .into_iter()
                    .map(|m| {
                        if let Some(meaning_info) = &m.meaning_info {
                            format!("{} ({meaning_info})", m.meaning)
                        } else {
                            m.meaning
                        }
                    })
                    .collect::<Vec<_>>();

                let text_reading = if i.text_word == i.text_reading_hiragana {
                    None
                } else {
                    Some(i.text_reading_hiragana)
                };
                let db_reading = if i.db_word == i.db_reading_hiragana {
                    None
                } else {
                    Some(i.db_reading_hiragana)
                };

                *field_id.borrow_mut() += 1;
                FormWordOr::FormWord(FormWord {
                    field_id: *field_id.borrow(),
                    word_id,
                    range: s_range.clone(),
                    text_word,
                    text_reading,
                    db_word: i.db_word,
                    db_reading,
                    score: i.score,
                    meanings,
                    tail: None,
                })
            })
        })
        .collect::<Vec<_>>();

    tracing::info!("fws {form_words:#?}");

    // group words by the starting index
    let mut grouped = Vec::new();
    let mut form_words = form_words.into_iter().peekable();
    loop {
        tracing::info!("next up {:#?}", form_words.peek());
        if let Some(peek) = form_words.peek() {
            let peek_start = peek.range().start;
            let mut segs = form_words
                .by_ref()
                .peeking_take_while(move |s| s.range().start == peek_start)
                .collect::<Vec<_>>();
            tracing::info!("segs {segs:#?}");
            // tack on text that is there before the next group
            if let Some(next) = form_words.peek() {
                let next_start = next.range().start;
                for seg in &mut segs {
                    if let FormWordOr::FormWord(form_word) = seg {
                        let fw_end = form_word.range.end;
                        if fw_end < next_start {
                            form_word.tail = Some(sentence[fw_end..next_start].to_string());
                        }
                    }
                }
            } else {
                // last group
                for seg in &mut segs {
                    if let FormWordOr::FormWord(form_word) = seg {
                        let sentence_end = sentence.len();
                        let fw_end = form_word.range.end;
                        if fw_end < sentence_end {
                            form_word.tail = Some(sentence[fw_end..sentence_end].to_string());
                        }
                    }
                }
            }
            // high score first
            segs.sort_unstable_by(|a, b| match (a, b) {
                (FormWordOr::FormWord(a), FormWordOr::FormWord(b)) => {
                    a.score.cmp(&b.score).reverse().then_with(|| {
                        // 中 readings are usually scored the same but なか is by far the most common one
                        if a.db_word == "中" && b.db_word == "中" {
                            if a.text_reading.as_deref() == Some("なか") {
                                Ordering::Less
                            } else {
                                Ordering::Greater
                            }
                        } else {
                            Ordering::Equal
                        }
                    })
                }
                (FormWordOr::FormWord(_), _) => Ordering::Less,
                (_, FormWordOr::FormWord(_)) => Ordering::Greater,
                (_, _) => Ordering::Equal,
            });
            grouped.push(segs);
        } else {
            break;
        }
    }

    let form = RwSignal::new(Form::init(&grouped));

    let form_word_views = grouped
        .into_iter()
        .map(|fw| {
            let group_size = fw.len();
            let grouped_view = fw
                .into_iter()
                .map(|fw| match fw {
                    FormWordOr::FormWord(fw) => {
                        let fw_accept = fw.clone();
                        let fw_accept_reading = fw.clone();
                        let fw_decline = fw.clone();
                        let fw_ignore = fw.clone();
                        let fw_class = fw.clone();

                        let meanings_view = fw
                            .meanings
                            .into_iter()
                            .map(|m| {
                                view! {
                                    <div>{m}</div>
                                }
                            })
                            .collect_view();
                        let accept_button = move || {
                            let accepted = form.read().is_accepted(fw_accept.field_id);
                            let fw_accept = fw_accept.clone();
                            let accept = move |_ev| form.write().accept(fw_accept.clone());
                            view! {
                                <button
                                    class="button"
                                    disabled={accepted}
                                    on:click={accept}
                                >
                                    "Accept"
                                </button>
                            }
                        };
                        let accept_reading_button = move || {
                            let accepted_reading = form.read().is_accepted_reading(fw_accept_reading.field_id);
                            let fw_accept_reading = fw_accept_reading.clone();
                            let accept_reading = move |_ev| form.write().accept_reading(fw_accept_reading.clone());
                            view! {
                                <button
                                    class="button"
                                    disabled={accepted_reading}
                                    on:click={accept_reading}
                                >
                                    "Accept Reading"
                                </button>
                            }
                        };
                        let decline_button = move || {
                            let declined = form.read().is_declined(fw_decline.field_id, fw_decline.word_id);
                            let fw_decline = fw_decline.clone();
                            view! {
                                <button
                                    class="button"
                                    disabled={declined}
                                    on:click={move |_ev| form.write().decline(fw_decline.clone())}
                                >
                                    "Decline"
                                </button>
                            }
                        };
                        let ignore_button = move || {
                            let ignored = form.read().is_ignored(fw_ignore.word_id);
                            let fw_ignore = fw_ignore.clone();
                            view! {
                                <button
                                    class="button"
                                    disabled={ignored}
                                    on:click={move |_ev| form.write().ignore(fw_ignore.clone())}
                                >
                                    "Ignore"
                                </button>
                            }
                        };
                        let word = if fw.db_reading.is_some() {
                            view! {
                                <div>{fw.db_word} " (" {fw.db_reading} ") [" {fw.score} "]"</div>
                            }.into_any()
                        } else {
                            view! {
                                <div>{fw.db_word} " [" {fw.score} "]"</div>
                            }.into_any()
                        };
                        view! {
                            <div
                                class="box is-flex is-flex-direction-column"
                                class:has-background-success-light=move || form.read().is_accepted(fw_class.field_id)
                                class:has-background-warning-light=move || !form.read().is_accepted(fw_class.field_id)
                                class:has-background-info-light=move || form.read().is_ignored(fw_class.word_id)
                            >
                                <div><b>{fw.text_word}</b>{fw.tail}</div>
                                {word}
                                <div>{meanings_view}</div>
                                <div>
                                    {accept_button}
                                    {accept_reading_button}
                                    {decline_button}
                                    {ignore_button}
                                </div>
                            </div>
                        }.into_any()
                    },
                    FormWordOr::Ignored { text_word, .. } => {
                        if group_size == 1 {
                            view! {
                                <div>{text_word} (ignored)</div>
                            }
                            .into_any()
                        } else {
                            ().into_any()
                        }
                    }
                    FormWordOr::Unknown { text_word, .. } => {
                        if group_size == 1 {
                            view! {
                                <div>{text_word} (unknown)</div>
                            }
                            .into_any()
                        } else {
                            ().into_any()
                        }
                    }
                })
                .collect_view();

            view! {
                <div class="columns is-flex is-flex-direction-row">
                    {grouped_view}
                </div>
            }
        })
        .collect_view();

    let accept_sentence = sentence.clone();
    let accept_sentence = Action::new(move |_| {
        let client = get_client();
        let req = form.get().finish(accept_sentence.clone());
        tracing::info!("finished");
        let on_successful_accept = on_successful_accept.clone();
        async move {
            if let Some(sentence_id) = sentence_id {
                SendWrapper::new(client.update_sentence(sentence_id, &req)).await?
            } else {
                SendWrapper::new(client.new_sentence(source_id, &req)).await?
            }
            on_successful_accept();
            WebResult::Ok(())
        }
    });
    let accept_result = move || match accept_sentence.value().get() {
        Some(Ok(())) => view! {
            <div>"success"</div>
        }
        .into_any(),
        Some(Err(err)) => view! {
            <div>"error " {err.to_string()}</div>
        }
        .into_any(),
        None => view! {
            <div></div>
        }
        .into_any(),
    };

    view! {
        <div class="block">
            <div id="paragraph-segmentation" class="subtitle" style="overflow-x:auto;">"Paragraph segmentation"</div>
            <div class="block">{sentence}</div>
            {form_word_views}
            <button class="button is-primary" on:click=move |_ev| { accept_sentence.dispatch(&()); }>"Accept sentence"</button>
            {accept_result}
        </div>
    }
}

#[derive(Debug, Clone)]
struct Form {
    accepted: Vec<FormWord>,
    accepted_readings: Vec<FormWord>,
    ignore_words: HashSet<i32>,
}

impl Form {
    fn init(form_words: &[Vec<FormWordOr>]) -> Self {
        let mut accepted = Vec::new();
        let mut covered_idx = 0;
        for form_word in form_words.iter().flat_map(|fw| fw.iter()) {
            if let FormWordOr::FormWord(form_word) = form_word {
                if form_word.range.start >= covered_idx {
                    // accept most likely interpretation
                    accepted.push(form_word.clone());
                    covered_idx = form_word.range.end;
                }
            }
        }
        Self {
            accepted,
            accepted_readings: Vec::new(),
            ignore_words: HashSet::new(),
        }
    }

    fn clear_accepted_range(&mut self, range: Range<usize>) {
        self.accepted
            .retain(|fw| fw.range.start >= range.end || fw.range.end <= range.start);
        self.accepted_readings
            .retain(|fw| fw.range.start >= range.end || fw.range.end <= range.start);
    }

    fn clear_by_field_id(&mut self, field_id: i32) {
        self.accepted.retain(|fw| fw.field_id != field_id);
        self.accepted_readings.retain(|fw| fw.field_id != field_id);
    }

    fn clear_by_word_id(&mut self, word_id: i32) {
        self.accepted.retain(|fw| fw.word_id != word_id);
        self.accepted_readings.retain(|fw| fw.word_id != word_id);
    }

    fn is_accepted(&self, field_id: i32) -> bool {
        self.accepted.iter().any(|fw| fw.field_id == field_id)
    }

    fn accept(&mut self, form_word: FormWord) {
        tracing::info!("Accepting {}", form_word.field_id);
        // un-accept all conflicting interpretations
        self.clear_accepted_range(form_word.range.clone());
        self.accepted.push(form_word);
    }

    fn is_accepted_reading(&self, field_id: i32) -> bool {
        self.accepted_readings
            .iter()
            .any(|fw| fw.field_id == field_id)
    }

    fn accept_reading(&mut self, form_word: FormWord) {
        tracing::info!("Accepting reading {}", form_word.field_id);
        // un-accept all conflicting interpretations
        self.clear_accepted_range(form_word.range.clone());
        self.accepted_readings.push(form_word);
    }

    fn is_ignored(&self, word_id: i32) -> bool {
        self.ignore_words.contains(&word_id)
    }

    fn ignore(&mut self, form_word: FormWord) {
        tracing::info!("Ignoring {}", form_word.word_id);
        self.clear_by_word_id(form_word.word_id);
        self.ignore_words.insert(form_word.word_id);
    }

    fn is_declined(&self, field_id: i32, word_id: i32) -> bool {
        !self.is_accepted(field_id)
            && !self.is_accepted_reading(field_id)
            && !self.is_ignored(word_id)
    }

    fn decline(&mut self, form_word: FormWord) {
        tracing::info!("Declining {}", form_word.field_id);
        self.clear_by_field_id(form_word.field_id);
        self.ignore_words.remove(&form_word.word_id);
    }

    fn finish(&self, sentence: String) -> req::SegmentedSentence {
        let words = self
            .accepted
            .iter()
            .map(|a| req::Word {
                id: Some(a.word_id),
                idx_start: a.range.start as i32,
                idx_end: a.range.end as i32,
                reading: a.text_reading.clone(),
            })
            .chain(self.accepted_readings.iter().map(|a| req::Word {
                id: None,
                idx_start: a.range.start as i32,
                idx_end: a.range.end as i32,
                reading: a.text_reading.clone(),
            }))
            .collect();
        req::SegmentedSentence {
            sentence,
            words,
            ignore_words: self.ignore_words.clone(),
        }
    }
}
