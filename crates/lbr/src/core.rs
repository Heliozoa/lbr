//! Contains functionality related to lbr_core.

use lbr_core::ichiran_types as it;
use std::collections::HashMap;

/// Converts ichiran segments to lbr's format.
///
/// # Panics
/// On some invalid ichiran inputs.
pub fn to_lbr_segments(
    text: &str,
    ichiran_segments: Vec<ichiran::Segment>,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
) -> Vec<it::Segment> {
    let mut segments = vec![];
    let mut idx = 0;
    for segment in ichiran_segments {
        if let ichiran::Segment::Words(words) = segment {
            for word_segment in words {
                for word in word_segment.words {
                    process_word(&mut segments, text, word, &mut idx, ichiran_word_to_id);
                }
            }
        }
    }
    // add the rest of the sentence as misc text
    if idx < text.len() {
        segments.push(it::Segment::Other(text[idx..].to_string()));
    }
    segments
}

fn process_word(
    segments: &mut Vec<it::Segment>,
    text: &str,
    word: ichiran::Word,
    idx: &mut usize,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
) {
    // handle word
    let mut word_in_text = None;
    let mut interpretations = vec![];
    for alternative in word.alternatives {
        let mut components = vec![];
        match alternative {
            ichiran::Alternative::WordInfo(info) => {
                let score = info.score;
                // replace zero width spaces
                let reading_hiragana = replace_invisible_characters(&info.kana);
                let component = to_lbr_word_info(info, ichiran_word_to_id);
                word_in_text = Some(component.word.clone());
                components.push(component);
                interpretations.push(it::Interpretation {
                    score,
                    reading_hiragana,
                    components,
                });
            }
            ichiran::Alternative::CompoundWordInfo(info) => {
                word_in_text = Some(info.text);
                // replace zero width spaces
                let reading_hiragana = replace_invisible_characters(&info.kana);
                for component in info.components {
                    let component = to_lbr_word_info(component, ichiran_word_to_id);
                    components.push(component);
                }
                interpretations.push(it::Interpretation {
                    score: info.score,
                    reading_hiragana,
                    components,
                });
            }
        };
    }

    // handle other segment between this and the previous word segment
    let word_in_text = word_in_text.unwrap();
    let (word_start_idx, length_in_target) =
        match lbr_core::find_jp_equivalent(&text[*idx..], &word_in_text) {
            Some(res) => res,
            None => {
                tracing::warn!(
                    "Failed to find word '{word_in_text}' from ichiran in text '{}'",
                    &text[*idx..]
                );
                return;
            }
        };
    if word_start_idx != 0 {
        let other = text[*idx..*idx + word_start_idx].to_string();
        segments.push(it::Segment::Other(other));
        *idx += word_start_idx;
    }
    *idx += length_in_target;

    segments.push(it::Segment::Phrase {
        phrase: word_in_text,
        interpretations,
    })
}

fn to_lbr_word_info(
    info: ichiran::WordInfo,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
) -> it::WordInfo {
    // we convert the ichiran seqs to our word ids here so we don't have to worry about them later
    let word_id = if let Some(seq) = dbg!(info.seq) {
        // ichiran's "reading" field contains the dictionary form e.g. "見る [みる]"
        if let Some((Some(first), second)) = info
            // get first conjunction
            .conj
            .first()
            // get reading from conjunction or from via if there is none
            .and_then(|c| {
                c.reading
                    .as_ref()
                    .or_else(|| c.via.first().and_then(|v| v.reading.as_ref()))
            })
            .map(|r| r.split_whitespace())
            .map(|mut ws| (ws.next(), ws.next()))
        {
            let dictionary_form = first.to_string();
            let reading = second
                .map(|s| s.replace("【", "").replace("】", ""))
                .unwrap_or_else(|| dictionary_form.clone());
            dbg!(ichiran_word_to_id
                .get(&(seq, dictionary_form, reading))
                .copied())
        } else {
            // if we can't find the dictionary form, we'll try with the word in text...
            ichiran_word_to_id
                .get(&(seq, info.text.clone(), info.kana.clone()))
                .copied()
                .or_else(|| {
                    // lastly we'll try with the reading
                    ichiran_word_to_id
                        .get(&(seq, info.kana.clone(), info.kana.clone()))
                        .copied()
                })
        }
    } else {
        None
    };
    // replace zero width spaces
    let reading_hiragana = replace_invisible_characters(&info.kana);
    it::WordInfo {
        word: info.text,
        reading_hiragana,
        word_id,
        meanings: info
            .gloss
            .into_iter()
            .chain(info.conj.into_iter().flat_map(|c| {
                c.gloss
                    .into_iter()
                    .chain(c.via.into_iter().flat_map(|v| v.gloss))
            }))
            .map(|g| it::Meaning {
                meaning: g.gloss,
                meaning_info: g.info,
            })
            .chain(info.suffix.map(|s| it::Meaning {
                meaning: s,
                meaning_info: None,
            }))
            .collect(),
    }
}

fn replace_invisible_characters(s: &str) -> String {
    s.replace("\u{200b}", "").replace("\u{200c}", "")
}

#[cfg(test)]
mod test {
    use super::*;

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
}
