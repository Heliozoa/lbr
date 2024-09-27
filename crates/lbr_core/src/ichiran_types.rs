//! Contains LBR types that represent (most of) the same data as the raw CLI output JSON from ichiran.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A segment of text, a single word or punctuation etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Segment {
    Phrase {
        /// The phrase as it appears in the text.
        phrase: String,
        /// Possible interpretations.
        interpretations: Vec<Interpretation>,
    },
    /// Non-word text segment, punctuation etc.
    Other(String),
}

impl Segment {
    /// Iterates over the seqs in this segment.
    pub fn seqs(&self) -> HashSet<i32> {
        let mut set = HashSet::new();
        if let Self::Phrase {
            interpretations, ..
        } = self
        {
            for interpretation in interpretations {
                for component in &interpretation.components {
                    if let Some(id) = component.word_id {
                        set.insert(id);
                    }
                }
            }
        }
        set
    }
}

/// A possible interpretation for a word.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interpretation {
    /// The higher the score, the more likely this is the correct interpretation according to ichiran.
    pub score: i32,
    /// The reading of the interpretation.
    pub reading_hiragana: String,
    /// A list of components that the phrase consists of.
    pub components: Vec<WordInfo>,
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
