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
    DeckSourceKind {
        Word: "word",
        Kanji: "kanji"
    }
);
