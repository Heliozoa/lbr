//! Provides all of LBR's core functionality.

pub mod anki;
pub mod core;
pub mod sentence_splitter;

fn is_kanji(c: char) -> bool {
    // Unicode CJK Unified Ideographs
    (0x4E00..0x9FBF).contains(&(c as u32))
}

pub fn kanji_from_word(word: &str) -> impl Iterator<Item = &str> {
    word.char_indices()
        .filter(|(_, c)| crate::is_kanji(*c))
        .map(|(i, c)| &word[i..i + c.len_utf8()])
}

pub struct StandardisedReading {
    pub hiragana: String,
    pub standardised: String,
}

pub fn standardise_reading(text: &str) -> StandardisedReading {
    use wana_kana::ConvertJapanese;
    let hiragana = text.to_hiragana();
    let standardised = hiragana.replace("づ", "ず").replace("ぢ", "じ");
    StandardisedReading {
        hiragana,
        standardised,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn recognises_kanji() {
        assert!(!is_kanji('k'));
        assert!(!is_kanji('え'));
        assert!(is_kanji('考'));
    }
}
