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
    ichiran_seq_to_word_id: &HashMap<i32, i32>,
) -> Vec<it::Segment> {
    let mut segments = vec![];
    let mut idx = 0;
    for segment in ichiran_segments {
        if let ichiran::Segment::Words(words) = segment {
            for word_segment in words {
                for word in word_segment.words {
                    process_word(&mut segments, text, word, &mut idx, ichiran_seq_to_word_id);
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
    ichiran_seq_to_word_id: &HashMap<i32, i32>,
) {
    // handle word
    let mut word_in_text = None;
    let mut interpretations = vec![];
    for alternative in word.alternatives {
        let mut components = vec![];
        match alternative {
            ichiran::Alternative::WordInfo(info) => {
                let score = info.score;
                let reading_hiragana = info.kana.clone();
                let component = to_lbr_word_info(info, ichiran_seq_to_word_id);
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
                let reading_hiragana = info.kana.clone();
                for component in info.components {
                    let component = to_lbr_word_info(component, ichiran_seq_to_word_id);
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
    let word_start_idx = text[*idx..].find(&word_in_text).unwrap();
    if word_start_idx != 0 {
        let other = text[*idx..*idx + word_start_idx].to_string();
        segments.push(it::Segment::Other(other));
        *idx += word_start_idx;
    }
    *idx += word_in_text.len();

    segments.push(it::Segment::Phrase {
        phrase: word_in_text,
        interpretations,
    })
}

fn to_lbr_word_info(
    info: ichiran::WordInfo,
    ichiran_seq_to_word_id: &HashMap<i32, i32>,
) -> it::WordInfo {
    // we convert the ichiran seqs to our word ids here so we don't have to worry about them later
    let word_id = info
        .seq
        .and_then(|seq| ichiran_seq_to_word_id.get(&seq).copied());
    it::WordInfo {
        word: info.text,
        reading_hiragana: info.kana,
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
