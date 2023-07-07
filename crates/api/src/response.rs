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
pub struct SourceWithSentences {
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
pub struct Sentence {
    pub id: i32,
    pub sentence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentedSentence {
    pub sentence: String,
    pub segments: Vec<Segment>,
    pub ignored_words: HashSet<i32>,
}
