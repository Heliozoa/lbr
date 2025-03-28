//! Contains functionality related to lbr_core.

use crate::{is_kanji, standardise_reading};
use lbr_core::ichiran_types as it;
use std::{collections::HashMap, ops::Not};

/// Converts ichiran segments to lbr's format.
///
/// # Panics
/// On some invalid ichiran inputs.
pub fn to_lbr_segments(
    text: &str,
    ichiran_segments: Vec<ichiran::Segment>,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
    kanji_to_readings: &HashMap<String, Vec<String>>,
    word_to_meanings: &HashMap<i32, Vec<String>>,
) -> Vec<it::Segment> {
    let mut segments = vec![];
    let mut idx = 0;
    for segment in ichiran_segments {
        if let ichiran::Segment::Segmentations(mut segmentations) = segment {
            // todo: process alternate segmentations
            segmentations.sort_by(|a, b| a.score.cmp(&b.score).reverse());
            if let Some(segmentation) = segmentations.into_iter().next() {
                tracing::trace!("Segmented {segmentation:#?}");
                for word in segmentation.words {
                    process_word(
                        &mut segments,
                        text,
                        word,
                        &mut idx,
                        ichiran_word_to_id,
                        kanji_to_readings,
                        word_to_meanings,
                    );
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
    kanji_to_readings: &HashMap<String, Vec<String>>,
    word_to_meanings: &HashMap<i32, Vec<String>>,
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
                let component = to_lbr_word_info(
                    info,
                    ichiran_word_to_id,
                    kanji_to_readings,
                    word_to_meanings,
                );
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
                    let component = to_lbr_word_info(
                        component,
                        ichiran_word_to_id,
                        kanji_to_readings,
                        word_to_meanings,
                    );
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
    kanji_to_readings: &HashMap<String, Vec<String>>,
    word_to_meanings: &HashMap<i32, Vec<String>>,
) -> it::WordInfo {
    // we convert the ichiran seqs to our word ids here so we don't have to worry about them later
    let word_id = if let Some(seq) = info.seq {
        // first, we'll try with the base info reading
        try_get_word_id(
            seq,
            &info.text,
            Some(&info.reading),
            ichiran_word_to_id,
            kanji_to_readings,
        )
        .or_else(|| {
            // second, we'll try the first conjugation...
            let conj_reading = info
                .conj
                .first()
                .and_then(|c| {
                    c.reading
                        .as_ref()
                        .or_else(|| c.via.first().and_then(|v| v.reading.as_ref()))
                })
                .map(String::as_str);
            try_get_word_id(
                seq,
                &info.text,
                conj_reading,
                ichiran_word_to_id,
                kanji_to_readings,
            )
        })
        /*
            .or_else(|| {
                // then with the word in text...
                try_get_word_id(Some(&info.text), ichiran_word_to_id)
            })
            .or_else(|| {
                // lastly we'll try with the reading...
                ichiran_word_to_id
                    .get(&(seq, info.kana.clone(), info.kana.clone()))
                    .copied()
            });
        */
    } else {
        None
    };
    // replace zero width spaces
    let reading_hiragana = replace_invisible_characters(&info.kana);
    if word_id.is_none() {
        tracing::debug!("Failed to find word id for {info:#?}");
    }
    let meanings = word_id
        .and_then(|wid| word_to_meanings.get(&wid))
        .map(|v| {
            v.iter()
                .map(|m| it::Meaning {
                    meaning: m.clone(),
                    meaning_info: None,
                })
                .collect()
        })
        .unwrap_or_else(|| {
            info.gloss
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
                .collect()
        });
    it::WordInfo {
        word: info.text,
        reading_hiragana,
        word_id,
        meanings,
    }
}

fn try_get_word_id(
    seq: i32,
    text: &str,
    reading: Option<&str>,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
    kanji_to_readings: &HashMap<String, Vec<String>>,
) -> Option<i32> {
    if let Some(reading) = reading {
        // ichiran's "reading" field contains the dictionary form e.g. "見る [みる]"
        let reading = replace_invisible_characters(reading);
        let mut split = reading.split_whitespace();
        if let (Some(first), second) = (split.next(), split.next()) {
            let dictionary_form_reading = second
                .map(|s| s.replace("【", "").replace("】", ""))
                .unwrap_or_else(|| first.to_string());
            let reading_standard = standardise_reading(&dictionary_form_reading);
            let dictionary_form = if text.chars().any(is_kanji) {
                first.to_string()
            } else {
                // if the word in text has no kanji, we'll use the reading as the dictionary form
                // because ichiran will return 為る for なる etc., and we want なる there
                // this is not a waterproof solution but should work well enough
                dictionary_form_reading.clone()
            };
            tracing::info!(
                "trying {} {} {}",
                seq,
                dictionary_form,
                reading_standard.standardised,
            );
            return ichiran_word_to_id
                .get(&(seq, dictionary_form.clone(), reading_standard.standardised))
                .or_else(|| {
                    if dictionary_form.chars().any(is_digit).not() {
                        return None;
                    }
                    // sometimes the "dictionary form" we find includes numbers before the actual word,
                    // for example for ２１度, 4日 etc.
                    // so if we fail to find the word we'll try to remove the numbers and try again...
                    let mut dictionary_form_without_numbers = String::new();
                    let mut reading_without_numbers = String::new();
                    if let Some(segmentation) = furigana::map(
                        &dictionary_form,
                        &dictionary_form_reading,
                        kanji_to_readings,
                    )
                    .iter()
                    .max_by_key(|f| f.accuracy)
                    {
                        for furigana in segmentation
                            .furigana
                            .iter()
                            // skip digit sections
                            .skip_while(|f| f.segment.chars().all(is_digit))
                        {
                            dictionary_form_without_numbers.push_str(furigana.segment);
                            reading_without_numbers
                                .push_str(furigana.furigana.unwrap_or(furigana.segment));
                        }
                    }

                    tracing::info!(
                        "trying {} {} {}",
                        seq,
                        dictionary_form_without_numbers,
                        reading_without_numbers,
                    );
                    ichiran_word_to_id.get(&(
                        seq,
                        dictionary_form_without_numbers,
                        reading_without_numbers,
                    ))
                })
                .or_else(|| {
                    if !dictionary_form.ends_with("目") && !dictionary_form.ends_with("間") {
                        return None;
                    }
                    // same as above but without 目 and 間 at the end to account for 人目 and　年間...
                    let mut dictionary_form_without_numbers = String::new();
                    let mut reading_without_numbers = String::new();
                    if let Some(segmentation) = furigana::map(
                        &dictionary_form,
                        &dictionary_form_reading,
                        kanji_to_readings,
                    )
                    .iter()
                    .max_by_key(|f| f.accuracy)
                    {
                        for furigana in segmentation
                            .furigana
                            .iter()
                            // skip digit sections
                            .skip_while(|f| f.segment.chars().all(is_digit))
                            .take_while(|f| f.segment != "目")
                            .take_while(|f| f.segment != "間")
                        {
                            dictionary_form_without_numbers.push_str(furigana.segment);
                            reading_without_numbers
                                .push_str(furigana.furigana.unwrap_or(furigana.segment));
                        }
                    }

                    tracing::info!(
                        "trying {} {} {}",
                        seq,
                        dictionary_form_without_numbers,
                        reading_without_numbers,
                    );
                    ichiran_word_to_id.get(&(
                        seq,
                        dictionary_form_without_numbers,
                        reading_without_numbers,
                    ))
                })
                .copied();
        }
    }
    None
}

fn replace_invisible_characters(s: &str) -> String {
    s.replace("\u{200b}", "").replace("\u{200c}", "")
}

fn is_digit(c: char) -> bool {
    matches!(
        c,
        '0' | '1'
            | '2'
            | '3'
            | '4'
            | '5'
            | '6'
            | '7'
            | '8'
            | '9'
            | '０'
            | '１'
            | '２'
            | '３'
            | '４'
            | '５'
            | '６'
            | '７'
            | '８'
            | '９'
            | '霊'
            | '一'
            | '二'
            | '三'
            | '四'
            | '五'
            | '六'
            | '七'
            | '八'
            | '九'
            | '十'
            | '百'
            | '千'
            | '万'
            | '億'
            | '兆'
            | '京'
    )
}
