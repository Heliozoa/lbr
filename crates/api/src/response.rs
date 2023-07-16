pub use chrono::{DateTime, Utc};
pub use lbr_core::ichiran_types::{Interpretation, Meaning, Segment, WordInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDetails {
    pub id: i32,
    pub name: String,
    pub sentences: Vec<Sentence>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckDetails {
    pub id: i32,
    pub name: String,
    pub sources: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoredWord {
    pub word_id: i32,
    pub translations: Vec<String>,
    pub written_forms: Vec<IgnoredWordWrittenForm>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoredWordWrittenForm {
    pub written_form: String,
    pub readings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sentence {
    pub id: i32,
    pub sentence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceDetails {
    pub id: i32,
    pub sentence: String,
    pub words: Vec<SentenceWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceWord {
    pub reading: Option<String>,
    pub idx_start: i32,
    pub idx_end: i32,
    pub furigana: Vec<Furigana>,
    pub translations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Furigana {
    pub word_start_idx: i32,
    pub word_end_idx: i32,
    pub reading_start_idx: i32,
    pub reading_end_idx: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedSentence {
    pub sentence: String,
    pub segments: Vec<Segment>,
    pub ignored_words: HashSet<i32>,
}
