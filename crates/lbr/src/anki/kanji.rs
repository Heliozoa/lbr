//! Kanji Anki cards.

use reanki::{Field, Model, Note, Template};
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub struct KanjiCard {
    pub id: i32,
    pub kanji: String,
    pub name: String,
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
            id: self.id.to_string(),
            count: self.kanji_words.to_string(),
            kanji,
            name,
            example_source_word,
            example_source_word_translation,
            similar_kanji,
            generated_at: std::time::UNIX_EPOCH
                .elapsed()
                .unwrap()
                .as_secs()
                .to_string(),
        }
    }

    pub fn into_note(self, model: Arc<Model>, template: Arc<Template>, order: u16) -> Note {
        // negate id to avoid conflicts with word ids
        let kanji_id = self.id;
        let guid: String = format!("lbr-kanji-{kanji_id}");
        let fields = self.into_fields();
        Note::new(guid, model, vec![template], fields.to_fields()).order(order)
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
    id: String,
    count: String,
    kanji: String,
    name: String,
    example_source_word: String,
    example_source_word_translation: String,
    similar_kanji: String,
    generated_at: String,
}

impl KanjiFields {
    // keep in sync with `to_fields`
    fn fields() -> Vec<Field> {
        vec![
            Field::new("id".to_string()),
            // the count should be the 1th field
            // as this is used by the model as the sort field
            Field::new("count".to_string()),
            Field::new("kanji".to_string()),
            Field::new("name".to_string()),
            Field::new("example_source_word".to_string()),
            Field::new("example_source_word_translation".to_string()),
            Field::new("similar_kanji".to_string()),
            Field::new("generated_at".to_string()),
        ]
    }

    // keep in sync with `fields`
    fn to_fields(self) -> Vec<String> {
        vec![
            self.id,
            self.count,
            self.kanji,
            self.name,
            self.example_source_word,
            self.example_source_word_translation,
            self.similar_kanji,
            self.generated_at,
        ]
    }
}

/// Globally unique anki model ID. Randomly chosen.
const LBR_KANJI_ANKI_MODEL_ID: i32 = -1289186172;
pub fn create_model() -> Model {
    let fields = KanjiFields::fields();
    Model::new(
        LBR_KANJI_ANKI_MODEL_ID,
        "lbr-kanji".to_string(),
        fields,
        1,
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
    font-size: 2rem;
}
#translation, #example {
    font-size: 2rem;
    display: inline-block;
    text-align: left;
}
"#
        .to_string(),
        reanki::ModelType::Standard,
    )
}

const LBR_KANJI_ANKI_TEMPLATE_ID: i32 = -155074387;
pub fn create_template() -> Template {
    Template::new(
        LBR_KANJI_ANKI_TEMPLATE_ID,
        "lbr-kanji".to_string(),
        r#"
<div id=kanji>
    {{kanji}}
</div>
"#
        .to_string(),
        r#"
<div id=answer>
    <div id=name>
        {{name}}
    </div>

    <hr>

    <div id=example>
        {{furigana:example_source_word}}
    </div>
    <div id=translation>
        {{example_source_word_translation}}
    </div>
</div>
"#
        .to_string(),
    )
}
