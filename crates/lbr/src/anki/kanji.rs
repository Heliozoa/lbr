//! Kanji Anki cards.

use genanki_rs::{Field, Model, Note, Template};

#[derive(Debug, PartialEq, Eq)]
pub struct KanjiCard {
    pub id: i32,
    pub kanji: String,
    pub name: Option<String>,
    pub example_source_word: KanjiWord,
    pub similar_kanji: Vec<Kanji>,
    pub kanji_words: usize,
}

impl KanjiCard {
    pub fn into_fields(self) -> KanjiFields {
        // kanji
        let kanji = self.kanji;

        // name
        let name = self.name;

        // example-source-word
        let example_source_word = self.example_source_word.word;

        // example-source-word-translation
        let example_source_word_translation = self.example_source_word.translations.join(", ");

        // similar-kanji
        let similar_kanji = self
            .similar_kanji
            .into_iter()
            .map(|k| {
                if let Some(name) = k.name {
                    format!("<li>{} ({})</li>", k.kanji, name)
                } else {
                    format!("<li>{}</li>", k.kanji)
                }
            })
            .collect::<Vec<_>>()
            .join("");
        let similar_kanji = format!("<ul>{similar_kanji}</ul>");

        KanjiFields {
            kanji,
            name,
            example_source_word,
            example_source_word_translation,
            similar_kanji,
        }
    }

    pub fn into_note(self, model: &Model) -> Note {
        // negate id to avoid conflicts with word ids
        let kanji_id = self.id;
        let guid = format!("lbr-kanji-{kanji_id}");
        let fields = self.into_fields();
        Note::new_with_options(model.clone(), fields.to_fields(), None, None, Some(&guid)).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct KanjiWord {
    pub word: String,
    pub translations: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Kanji {
    pub kanji: String,
    pub name: Option<String>,
}

/// Wrapper for the fields of an Anki card to make handling them in a typesafe way easier.
#[derive(Debug)]
pub struct KanjiFields {
    pub kanji: String,
    pub name: Option<String>,
    pub example_source_word: String,
    pub example_source_word_translation: String,
    pub similar_kanji: String,
}

impl KanjiFields {
    // keep in sync with `to_fields`
    fn fields() -> Vec<Field> {
        vec![
            Field::new("kanji"),
            Field::new("name"),
            Field::new("example-source-word"),
            Field::new("example-source-word-translation"),
            Field::new("similar-kanji"),
        ]
    }

    // keep in sync with `fields`
    fn to_fields(&self) -> Vec<&str> {
        vec![
            &self.kanji,
            &self.name.as_deref().unwrap_or(""),
            &self.example_source_word,
            &self.example_source_word_translation,
            &self.similar_kanji,
        ]
    }
}

/// Globally unique anki model ID. Randomly chosen.
const MODEL_ID: i64 = -1842913271028638742;
pub fn create_model() -> Model {
    let fields = KanjiFields::fields();
    let templates = vec![Template::new("lbr-kanji")
        .qfmt(
            r#"
<div id=kanji>
    {{kanji}}
</div>
"#,
        )
        .afmt(
            r#"
<div id=answer>
    <div id=name>
        {{name}}
    </div>

    <hr>

    <div id=example>
        {{furigana:example-source-word}}
    </div>
    <br/>
    <div id=translation>
        {{example-source-word-translation}}
    </div>
</div>
"#,
        )];
    Model::new(MODEL_ID, "lbr-kanji", fields, templates).css(
        r#"
.card {
    text-align: center;
    background-color: Linen;
    font-size: 1.5rem;
}
#highlighted {
    color: red;
}
#kanji {
    font-size: 4rem;
}
#translation, #example {
    font-size: 2rem;
    display: inline-block;
    text-align: left;
}
"#,
    )
}
