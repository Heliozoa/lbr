//! Types for responses from the backend to the frontend.

pub use chrono::{DateTime, Utc};
use lbr_core::ichiran_types;
pub use lbr_core::ichiran_types::{Meaning, Segment, WordInfo};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, ops::Range};

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
    pub sources: Vec<DeckSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckSource {
    pub id: i32,
    pub threshold: i32,
    pub kind: DeckSourceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeckSourceKind {
    Kanji,
    Word,
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
    pub source_id: i32,
    pub sentence: String,
    pub words: Vec<SentenceWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceWord {
    pub word: String,
    pub reading: String,
    pub sentence_word_reading: Option<String>,
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
pub struct SegmentedParagraph {
    pub sentences: Vec<SegmentedParagraphSentence>,
    pub ignored_words: HashSet<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedParagraphSentence {
    pub sentence: String,
    pub segments: Vec<ApiSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedSentence {
    pub sentence: String,
    pub segments: Vec<ApiSegment>,
    pub ignored_words: HashSet<i32>,
}

/// A segment of text, a single word or punctuation etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSegment {
    /// The segment as it appears in the text.
    pub text: String,
    /// List of potential interpretations for the text,
    /// empty for non-word text segments.
    pub interpretations: Vec<ApiInterpretation>,
    /// The range covered by this segment in the original text.
    pub range: Range<usize>,
}

/// A single interpretation for a segment of text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInterpretation {
    /// LBR word id.
    pub word_id: Option<i32>,
    /// The higher the score, the more likely this is the correct interpretation according to ichiran.
    pub score: i32,
    /// The word as it appears in the text.
    pub text_word: String,
    /// The reading of the word as it appears in the text.
    pub text_reading_hiragana: String,
    /// The word as it appears in the database.
    pub db_word: String,
    /// The reading of the word as it appears in the database.
    pub db_reading_hiragana: String,
    /// List of possible meanings for the word.
    pub meanings: Vec<ichiran_types::Meaning>,
}
