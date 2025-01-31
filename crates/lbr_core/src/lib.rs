//! LBR core types and functions.

use std::ops::Range;

pub mod ichiran_types;

// ichiran sometimes returns characters in a slightly different, equivalent form
// this function lets us ignore the differences
// returns the index and the length in the text
pub fn find_jp_equivalent(text: &str, target: &str) -> Option<(usize, usize)> {
    const UNICODE_KANA_TABLE_DISTANCE: u32 = 'ア' as u32 - 'あ' as u32;
    const UNICODE_NUM_TABLE_DISTANCE: u32 = '０' as u32 - '0' as u32;
    const HIRAGANA_RANGE: Range<char> = '\u{3041}'..'\u{3096}';
    const KATAKANA_RANGE: Range<char> = '\u{30A1}'..'\u{30F6}';
    const NUM_RANGE: Range<char> = '0'..'9';
    const FW_NUM_RANGE: Range<char> = '０'..'９';

    let mut text_idx = 0;
    let mut text_chars = text.chars();
    let mut target_length_in_text = 0;
    loop {
        if text[text_idx..].len() < target.len() {
            return None;
        }

        let mut mismatch = false;
        for (left, right) in text[text_idx..].chars().zip(target.chars()) {
            if left == right {
                // same char
                target_length_in_text += left.len_utf8();
                continue;
            }

            if HIRAGANA_RANGE.contains(&left) {
                if left as u32 == (right as u32).saturating_sub(UNICODE_KANA_TABLE_DISTANCE) {
                    // kana equivalent
                    target_length_in_text += left.len_utf8();
                    continue;
                }
            } else if KATAKANA_RANGE.contains(&left) {
                if left as u32 == (right as u32).saturating_add(UNICODE_KANA_TABLE_DISTANCE) {
                    // kana equivalent
                    target_length_in_text += left.len_utf8();
                    continue;
                }
            } else if NUM_RANGE.contains(&left) {
                if left as u32 == (right as u32).saturating_sub(UNICODE_NUM_TABLE_DISTANCE) {
                    // width equivalent
                    target_length_in_text += left.len_utf8();
                    continue;
                }
            } else if FW_NUM_RANGE.contains(&left) {
                if left as u32 == (right as u32).saturating_add(UNICODE_NUM_TABLE_DISTANCE) {
                    // width equivalent
                    target_length_in_text += left.len_utf8();
                    continue;
                }
            }

            mismatch = true;
            target_length_in_text = 0;
            break;
        }

        if !mismatch {
            return Some((text_idx, target_length_in_text));
        }

        if let Some(next) = text_chars.next() {
            text_idx += next.len_utf8();
        } else {
            break;
        }
    }
    None
}
