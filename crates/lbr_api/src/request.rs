//! Types for requests from the frontend to the backend.

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashSet};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Login<'a> {
    pub email: Cow<'a, str>,
    pub password: Cow<'a, str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Register<'a> {
    pub email: Cow<'a, str>,
    pub password: Cow<'a, str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewSource<'a> {
    pub name: Cow<'a, str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateSource<'a> {
    pub name: Cow<'a, str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewDeck<'a> {
    pub name: Cow<'a, str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateDeck<'a> {
    pub name: Cow<'a, str>,
    pub included_sources: Cow<'a, [IncludedSource]>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct IncludedSource {
    pub source_id: i32,
    pub threshold: i32,
    pub kind: IncludedSourceKind,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum IncludedSourceKind {
    Word,
    Kanji,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateSentence<'a> {
    pub sentence: Cow<'a, str>,
    pub sentence_words: Cow<'a, [UpdatedSentenceWord<'a>]>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdatedSentenceWord<'a> {
    pub word_id: i32,
    pub reading: Option<Cow<'a, str>>,
    pub idx_start: i32,
    pub idx_end: i32,
    pub furigana: Vec<UpdatedSentenceWordFurigana>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdatedSentenceWordFurigana {
    pub word_start_idx: i32,
    pub word_end_idx: i32,
    pub reading_start_idx: i32,
    pub reading_end_idx: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Paragraph<'a> {
    pub source_id: i32,
    pub paragraph: Cow<'a, str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SegmentedSentence {
    pub sentence: String,
    pub words: Vec<Word>,
    pub ignore_words: HashSet<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Word {
    pub id: Option<i32>,
    pub reading: Option<String>,
    pub idx_start: i32,
    pub idx_end: i32,
}
