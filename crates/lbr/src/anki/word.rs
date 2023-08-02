//! Sentence word Anki cards.

use genanki_rs::{Field, Model, Note, Template};
use serde::Deserialize;
use std::ops::Range;

#[derive(Debug, PartialEq, Eq)]
pub struct WordCard {
    pub id: i32,
    pub word: String,
    pub word_range: Range<usize>,
    pub word_furigana: Vec<Furigana>,
    pub word_sentences: usize,
    pub sentence: Sentence,
    pub translations: Vec<String>,
    pub kanji: Vec<WordKanji>,
}

impl WordCard {
    pub fn into_fields(self) -> WordFields {
        // count
        let count = self.word_sentences.to_string();

        // sentence
        // here, we insert furigana ruby and highlight the card's word in the sentence
        let mut sentence_idx = 0;
        let mut sentence = String::new();
        let sentence_text = &self.sentence.sentence;
        // process sentence words in order of appearance
        let mut sentence_words = self.sentence.words;
        sentence_words.sort_by_key(|sw| sw.idx_start);
        for sw in sentence_words {
            // insert stuff that's before this sentence word
            // sentence words don't cover punctuation etc., so this step is needed
            // to ensure it's accounted for
            let sentence_word_start = sw.idx_start as usize;
            if sentence_idx < sentence_word_start {
                let push = &sentence_text[sentence_idx..sentence_word_start];
                sentence.push_str(push);
                if !push.ends_with(char::is_whitespace) {
                    // empty ruby are used to prevent furigana from running over past where it's supposed to
                    sentence.push_str("[ ]");
                }
            }
            sentence_idx = sentence_word_start;

            // check if this sentence word is the word on the card
            let in_word = self.word_range.start == sentence_idx;
            if in_word {
                sentence.push_str(r#"<span id=highlighted>"#);
            }

            // process furigana in order of appearance
            let mut furigana = sw.furigana;
            furigana.sort_by_key(|f| f.range.start);
            for furigana in furigana {
                let furigana_start = sentence_word_start + furigana.range.start;
                let furigana_end = sentence_word_start + furigana.range.end;

                // push stuff before this furigana that wasn't processed yet
                if sentence_idx < furigana_start {
                    sentence.push_str(&sentence_text[sentence_idx..furigana_start]);
                    sentence.push_str("[ ]");
                }

                // push stuff covered by this furigana
                sentence.push_str(&sentence_text[furigana_start..furigana_end]);
                sentence.push_str(&format!("[{}]", furigana.furigana));
                sentence_idx = furigana_end;
            }

            // insert stuff left over after the furigana
            let sw_idx_end = sw.idx_end as usize;
            if sentence_idx < sw_idx_end {
                let push = &sentence_text[sentence_idx..sw_idx_end];
                sentence.push_str(push);
                if !push.ends_with(char::is_whitespace) {
                    sentence.push_str("[ ]");
                }
                sentence_idx = sw_idx_end;
            }
            if in_word {
                sentence.push_str("</span>");
            }
        }

        // push stuff left over after processing all sentence words
        if sentence_idx < self.sentence.sentence.len() {
            sentence.push_str(&sentence_text[sentence_idx..]);
        }

        // word
        let word = self.word;

        // translation
        let translation = self.translations.join("<br/>");

        // kanji
        let kanji = self
            .kanji
            .into_iter()
            .map(|k| {
                if let Some(name) = k.name {
                    format!("{} ({})", k.chara, name)
                } else {
                    k.chara
                }
            })
            .collect::<Vec<_>>()
            .join("<br />");
        // empty fields cause anki to think all such cards are identical
        let kanji = if kanji.is_empty() {
            " ".to_string()
        } else {
            kanji
        };

        WordFields {
            count,
            sentence,
            word,
            translation,
            kanji,
        }
    }

    pub fn into_note(self, model: &Model) -> Note {
        let word_id = self.id;
        let guid = format!("lbr-word-{word_id}");
        let fields = self.into_fields();
        Note::new_with_options(model.clone(), fields.to_fields(), None, None, Some(&guid)).unwrap()
    }
}

/// An example sentence included in an LBR Anki card.
#[derive(Debug, PartialEq, Eq)]
pub struct Sentence {
    pub sentence: String,
    pub words: Vec<SentenceWord>,
}

/// Delineates a word within a sentence with furigana, if any.
#[derive(Debug, PartialEq, Eq)]
pub struct SentenceWord {
    pub furigana: Vec<Furigana>,
    pub idx_start: i32,
    pub idx_end: i32,
}

/// Furigana for a sentence word.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Furigana {
    pub range: Range<usize>,
    pub furigana: String,
}

/// Kanji included in an Anki card word.
#[derive(Debug, PartialEq, Eq)]
pub struct WordKanji {
    pub chara: String,
    pub name: Option<String>,
}

/// Wrapper for the fields of an Anki card to make handling them in a typesafe way easier.
#[derive(Debug)]
pub struct WordFields {
    count: String,
    sentence: String,
    word: String,
    translation: String,
    kanji: String,
}

impl WordFields {
    // keep in sync with `to_fields`
    fn fields() -> Vec<Field> {
        vec![
            Field::new("count"),
            Field::new("sentence"),
            Field::new("word"),
            Field::new("translation"),
            Field::new("kanji"),
        ]
    }

    // keep in sync with `fields`
    fn to_fields(&self) -> Vec<&str> {
        vec![
            &self.count,
            &self.sentence,
            &self.word,
            &self.translation,
            &self.kanji,
        ]
    }
}

/// Globally unique anki model ID. Randomly chosen.
const MODEL_ID: i64 = -4236074849754614939;
pub fn create_model() -> Model {
    let fields = WordFields::fields();
    let templates = vec![Template::new("lbr-word")
        .qfmt(
            r#"
<div id=sentence>
    {{furigana:sentence}}
</div>
"#,
        )
        .afmt(
            r#"
<div id=answer>
    <div id=sentence>
        {{furigana:sentence}}
    </div>

    <hr>

    <div id=word>
        {{furigana:word}}
    </div>
    <div id=kanji>
        {{kanji}}
    </div>
    <br/>
    <div id=translation>
        {{translation}}
    </div>
</div>
"#,
        )];
    Model::new(MODEL_ID, "lbr-word", fields, templates).css(
        r#"
.card {
    text-align: center;
    background-color: Linen;
    font-size: 1.5rem;
}
#highlighted {
    color: red;
}
#word {
    font-size: 4rem;
}
#sentence, #translation, #kanji {
    font-size: 2rem;
}
#translation, #kanji {
    display: inline-block;
    text-align: left;
}
ruby rt {
    visibility: hidden;
}
#answer ruby rt {
    visibility: visible;
}
"#,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn creates_fields_from_card() {
        let card = WordCard {
            id: 1,
            word: "猫".to_string(),
            word_range: 9..12,
            word_furigana: vec![Furigana {
                range: 0..3,
                furigana: "ねこ".to_string(),
            }],
            word_sentences: 1,
            sentence: Sentence {
                sentence: "吾輩は猫である".to_string(),
                words: vec![
                    SentenceWord {
                        idx_start: 0,
                        idx_end: 6,
                        furigana: vec![
                            Furigana {
                                range: 0..3,
                                furigana: "わが".to_string(),
                            },
                            Furigana {
                                range: 3..6,
                                furigana: "はい".to_string(),
                            },
                        ],
                    },
                    SentenceWord {
                        idx_start: 6,
                        idx_end: 9,
                        furigana: vec![],
                    },
                    SentenceWord {
                        idx_start: 9,
                        idx_end: 12,
                        furigana: vec![Furigana {
                            range: 0..3,
                            furigana: "ねこ".to_string(),
                        }],
                    },
                    SentenceWord {
                        idx_start: 12,
                        idx_end: 15,
                        furigana: vec![],
                    },
                    SentenceWord {
                        idx_start: 15,
                        idx_end: 21,
                        furigana: vec![],
                    },
                ],
            },
            translations: vec!["Cat".to_string()],
            kanji: vec![WordKanji {
                chara: "猫".to_string(),
                name: Some("Cat".to_string()),
            }],
        };

        let fields = card.into_fields();
        assert_eq!(
            fields.sentence,
            "吾[わが]輩[はい]は[ ]<span id=highlighted>猫[ねこ]</span>で[ ]ある[ ]"
        );
    }
}
