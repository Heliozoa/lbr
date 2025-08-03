//! LBR core types and functions.

use std::ops::RangeInclusive;

pub mod ichiran_types;

// ichiran sometimes returns characters in a slightly different, equivalent form
// this function lets us ignore the differences
// returns the index and the length in the text
pub fn find_jp_equivalent(text: &str, target: &str) -> Option<(usize, usize)> {
    const UNICODE_KANA_TABLE_DISTANCE: u32 = 'ア' as u32 - 'あ' as u32;
    const UNICODE_NUM_TABLE_DISTANCE: u32 = '０' as u32 - '0' as u32;
    const HIRAGANA_RANGE: RangeInclusive<char> = '\u{3041}'..='\u{3096}';
    const KATAKANA_RANGE: RangeInclusive<char> = '\u{30A1}'..='\u{30F6}';
    const NUM_RANGE: RangeInclusive<char> = '0'..='9';
    const FW_NUM_RANGE: RangeInclusive<char> = '０'..='９';

    if target.is_empty() {
        return None;
    }

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

            let hiragana_equivalent = HIRAGANA_RANGE.contains(&left)
                && left as u32 == (right as u32).saturating_sub(UNICODE_KANA_TABLE_DISTANCE);
            let katakana_equivalent = KATAKANA_RANGE.contains(&left)
                && left as u32 == (right as u32).saturating_add(UNICODE_KANA_TABLE_DISTANCE);
            let num_equivalent = NUM_RANGE.contains(&left)
                && left as u32 == (right as u32).saturating_sub(UNICODE_NUM_TABLE_DISTANCE);
            let fw_equivalent = FW_NUM_RANGE.contains(&left)
                && left as u32 == (right as u32).saturating_add(UNICODE_NUM_TABLE_DISTANCE);
            if hiragana_equivalent || katakana_equivalent || num_equivalent || fw_equivalent {
                // kana equivalent
                target_length_in_text += left.len_utf8();
                continue;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn finds_number() {
        let res = find_jp_equivalent("今は９時です。", "9時");
        assert_eq!(res, Some((6, 6)));

        let res = find_jp_equivalent("今日は４日です。", "4日");
        assert_eq!(res, Some((9, 6)));
    }

    #[test]
    fn finds_regular() {
        let res = find_jp_equivalent("abcdefg", "def");
        assert_eq!(res, Some((3, 3)));
    }

    #[test]
    fn fails_to_find() {
        let res = find_jp_equivalent("abcdefg", "z");
        assert_eq!(res, None);
    }

    #[test]
    fn finds_kana_equivalent() {
        let res = find_jp_equivalent("そろそろ１０時間ですね", "デス");
        assert_eq!(res, Some((24, 6)));
    }

    #[test]
    fn finds_width_equivalent() {
        let res = find_jp_equivalent("そろそろ１０時間ですね", "10");
        assert_eq!(res, Some((12, 6)));
    }

    #[test]
    fn works_with_empty() {
        let res = find_jp_equivalent("asd", "");
        assert_eq!(res, None);
    }
}
