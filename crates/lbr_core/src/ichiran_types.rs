//! Contains LBR types that represent (most of) the same data as the raw CLI output JSON from ichiran.

use serde::{Deserialize, Serialize};
use std::ops::Range;

/// A segment of text, a single word or punctuation etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    /// The segment as it appears in the text.
    pub text: String,
    /// List of potential interpretations for the text,
    /// empty for non-word text segments.
    pub interpretations: Vec<Interpretation>,
    /// The range covered by this segment in the original text.
    pub range: Range<usize>,
}

/// A single interpretation for a segment of text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interpretation {
    /// LBR word id.
    pub word_id: Option<i32>,
    /// The higher the score, the more likely this is the correct interpretation according to ichiran.
    pub score: i32,
    /// The word as it appears in the text.
    pub word: String,
    /// The reading of the word as it appears in the text.
    pub reading_hiragana: String,
    /// List of possible meanings for the word.
    pub meanings: Vec<Meaning>,
}

/// Information for an interpretation of a single word.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordInfo {
    /// The word as it appears in the text.
    pub word: String,
    /// The reading of the word as it appears in the text.
    pub reading_hiragana: String,
    /// LBR word id.
    pub word_id: Option<i32>,
    /// List of possible meanings for the word.
    pub meanings: Vec<Meaning>,
}

/// The English meaning of a Japanese word.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meaning {
    /// An English translation of the word's meaning.
    pub meaning: String,
    /// Additional information regarding the usage of the word in this meaning.
    pub meaning_info: Option<String>,
}
