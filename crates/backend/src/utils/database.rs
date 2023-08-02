//! Rust types for custom database types.

use crate::utils::diesel::{diesel_enum, diesel_struct};

diesel_struct!(
    #[derive(Clone, Copy, PartialEq, Eq)]
    Furigana {
        word_start_idx: i32 = Integer,
        word_end_idx: i32 = Integer,
        reading_start_idx: i32 = Integer,
        reading_end_idx: i32 = Integer,
    }
);

diesel_enum!(
    #[derive(Clone, Copy, PartialEq, Eq)]
    Position: Pos {
        Prefix: "prefix",
        Suffix: "suffix"
    }
);

impl From<jadata::kanjifile::Position> for Position {
    fn from(value: jadata::kanjifile::Position) -> Self {
        match value {
            jadata::kanjifile::Position::Prefix => Self::Prefix,
            jadata::kanjifile::Position::Suffix => Self::Suffix,
        }
    }
}

diesel_enum!(
    #[derive(Clone, Copy, PartialEq, Eq)]
    ReadingKind {
        Onyomi: "onyomi",
        Kunyomi: "kunyomi"
    }
);

impl From<jadata::kanjifile::ReadingKind> for ReadingKind {
    fn from(value: jadata::kanjifile::ReadingKind) -> Self {
        match value {
            jadata::kanjifile::ReadingKind::Kunyomi => Self::Kunyomi,
            jadata::kanjifile::ReadingKind::Onyomi => Self::Onyomi,
        }
    }
}

diesel_enum!(
    #[derive(Clone, Copy, PartialEq, Eq)]
    DeckSourceKind {
        Word: "word",
        Kanji: "kanji"
    }
);
