//! Contains functionality related to lbr_core.

use crate::{is_kanji, standardise_reading, StandardisedReading};
use ichiran::{Alternative, WordInfo};
use lbr_core::ichiran_types::{self as it, Segment};
use std::{
    collections::{hash_map::Entry, HashMap},
    ops::{Not, Range},
};

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
    tracing::debug!("Processing segmentation for {text}");
    tracing::trace!("{ichiran_segments:#?}");

    let mut segments = vec![];
    let mut next_segment_start_idx = 0;
    // we can merge all segmentations over a specific range into one node
    let mut new_segments = HashMap::<Range<usize>, Segment>::new();
    for (idx, segment) in ichiran_segments.into_iter().enumerate() {
        let current_segment_start_idx = next_segment_start_idx;
        tracing::trace!("processing segment {idx}");
        match segment {
            ichiran::Segment::Segmentations(segmentations) => {
                for (idx, segmentation) in segmentations.into_iter().enumerate() {
                    tracing::trace!("processing segmentation {idx}");
                    let mut current_idx = current_segment_start_idx;
                    for word in segmentation.words {
                        let remaining_text = &text[current_idx..];
                        tracing::trace!(
                            "processing word, sidx {current_idx}, rem {remaining_text}"
                        );
                        let alternative_start_idx = current_idx;
                        let mut alternative_max_idx = current_idx;
                        for alternative in word.alternatives {
                            current_idx = alternative_start_idx;
                            match alternative {
                                Alternative::WordInfo(wi) => {
                                    if let Some(range) = process_word_info(
                                        wi,
                                        remaining_text,
                                        ichiran_word_to_id,
                                        kanji_to_readings,
                                        word_to_meanings,
                                        &mut new_segments,
                                        current_idx,
                                    ) {
                                        alternative_max_idx = alternative_max_idx.max(range.end);
                                        next_segment_start_idx =
                                            next_segment_start_idx.max(range.end);
                                    }
                                }
                                Alternative::CompoundWordInfo(cwi) => {
                                    for component in cwi.components {
                                        let remaining_text = &text[current_idx..];
                                        if let Some(range) = process_word_info(
                                            component,
                                            remaining_text,
                                            ichiran_word_to_id,
                                            kanji_to_readings,
                                            word_to_meanings,
                                            &mut new_segments,
                                            current_idx,
                                        ) {
                                            current_idx = range.end;
                                            alternative_max_idx =
                                                alternative_max_idx.max(range.end);
                                            next_segment_start_idx =
                                                next_segment_start_idx.max(range.end);
                                        }
                                    }
                                }
                            }
                        }
                        current_idx = alternative_max_idx;
                    }
                }
            }
            ichiran::Segment::Other(other) => {
                tracing::trace!("other {other}");
            }
        }
    }
    let mut new_segments = new_segments.into_values().collect::<Vec<_>>();
    new_segments.sort_unstable_by(|a, b| {
        a.range
            .start
            .cmp(&b.range.start)
            .then(a.range.end.cmp(&b.range.end))
    });
    new_segments.iter_mut().for_each(|ns| {
        ns.interpretations
            .sort_unstable_by(|a, b| a.score.cmp(&b.score).reverse())
    });
    for new_segment in new_segments {
        segments.push(new_segment);
    }
    segments
}

// returns the range of the word in the text
fn process_word_info(
    wi: WordInfo,
    remaining_text: &str,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
    kanji_to_readings: &HashMap<String, Vec<String>>,
    word_to_meanings: &HashMap<i32, Vec<String>>,
    new_segments: &mut HashMap<Range<usize>, Segment>,
    current_idx: usize,
) -> Option<Range<usize>> {
    let word = &wi.text;
    let Some((start_idx, len)) = lbr_core::find_jp_equivalent(remaining_text, word) else {
        tracing::warn!("Failed to find {word} in text {remaining_text}");
        return None;
    };
    let local_range = start_idx..(start_idx + len);
    let segment_range = local_range.start + current_idx..local_range.end + current_idx;
    let word_in_text = &remaining_text[local_range.clone()];
    tracing::trace!(
        "local range {local_range:?} in {remaining_text}, local result {word_in_text}, segment range {segment_range:?}",
    );
    let Some(seq) = wi.seq else {
        tracing::warn!("No seq for {word}");
        return Some(segment_range);
    };
    let word_id = try_get_word_id(
        seq,
        word_in_text,
        &wi.reading,
        ichiran_word_to_id,
        kanji_to_readings,
    )
    .or_else(|| {
        // try first conj
        wi.conj.get(0).and_then(|conj| {
            conj.reading
                .as_ref()
                .and_then(|reading| {
                    try_get_word_id(
                        seq,
                        word_in_text,
                        reading,
                        ichiran_word_to_id,
                        kanji_to_readings,
                    )
                })
                .or_else(|| {
                    // try first via if reading or conj word id is none
                    conj.via
                        .get(0)
                        .and_then(|via| via.reading.as_ref())
                        .and_then(|reading| {
                            try_get_word_id(
                                seq,
                                word_in_text,
                                reading,
                                ichiran_word_to_id,
                                kanji_to_readings,
                            )
                        })
                })
        })
    });
    if word_id.is_none() {
        tracing::warn!("Failed to find word_id for {}", wi.text);
    }

    let reading_hiragana = replace_invisible_characters(&wi.kana);
    let meanings = word_id
        // get from map
        .and_then(|wid| word_to_meanings.get(&wid))
        .map(|v| {
            v.iter()
                .map(|m| it::Meaning {
                    meaning: m.clone(),
                    meaning_info: None,
                })
                .collect()
        })
        // or otherwise from gloss
        .unwrap_or_else(|| {
            wi.gloss
                .into_iter()
                .chain(wi.conj.into_iter().flat_map(|c| {
                    c.gloss
                        .into_iter()
                        .chain(c.via.into_iter().flat_map(|v| v.gloss))
                }))
                .map(|g| it::Meaning {
                    meaning: g.gloss,
                    meaning_info: g.info,
                })
                .chain(wi.suffix.map(|s| it::Meaning {
                    meaning: s,
                    meaning_info: None,
                }))
                .collect()
        });
    let new_interpretation: it::Interpretation = it::Interpretation {
        word_id,
        score: wi.score,
        word: wi.text,
        reading_hiragana,
        meanings,
    };
    match new_segments.entry(segment_range.clone()) {
        Entry::Occupied(mut new_segment) => {
            let interpretations = &mut new_segment.get_mut().interpretations;
            if interpretations
                .iter()
                .all(|ni| ni.word_id != new_interpretation.word_id)
            {
                interpretations.push(new_interpretation);
            }
        }
        Entry::Vacant(vacant) => {
            let word_in_text = remaining_text
                .get(local_range.clone())
                .ok_or_else(|| {
                    format!("Failed to index into '{remaining_text}' with {local_range:?}")
                })
                .unwrap();
            vacant.insert(Segment {
                text: word_in_text.to_string(),
                interpretations: vec![new_interpretation],
                range: segment_range.clone(),
            });
        }
    };
    Some(segment_range)
}

fn parse_ichiran_reading(
    word_in_text: &str,
    ichiran_reading: &str,
) -> Option<(String, StandardisedReading)> {
    tracing::info!("parsing {word_in_text} {ichiran_reading}");
    // ichiran's "reading" field contains the dictionary form e.g. "見る [みる]"
    let reading = replace_invisible_characters(ichiran_reading);
    let mut split = reading.split_whitespace();
    if let (Some(first), second) = (split.next(), split.next()) {
        let dictionary_form_reading = second
            .map(|s| s.replace("【", "").replace("】", ""))
            .unwrap_or_else(|| first.to_string());
        let reading_standard = standardise_reading(&dictionary_form_reading);
        let dictionary_form = if word_in_text.chars().any(is_kanji) {
            first.to_string()
        } else {
            // if the word in text has no kanji, we'll use the reading as the dictionary form
            // because ichiran will return 為る for なる etc., and we want なる there
            // this is not a waterproof solution but should work well enough
            dictionary_form_reading.clone()
        };
        Some((dictionary_form, reading_standard))
    } else {
        None
    }
}

fn try_get_word_id(
    seq: i32,
    word_in_text: &str,
    ichiran_reading: &str,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
    kanji_to_readings: &HashMap<String, Vec<String>>,
) -> Option<i32> {
    let (dictionary_form, reading_standard) = parse_ichiran_reading(word_in_text, ichiran_reading)?;
    tracing::info!(
        "trying {} {} {}",
        seq,
        dictionary_form,
        reading_standard.standardised,
    );
    return ichiran_word_to_id
        .get(&(
            seq,
            dictionary_form.clone(),
            reading_standard.standardised.clone(),
        ))
        .or_else(|| {
            if dictionary_form.chars().any(is_digit).not() {
                return None;
            }
            // sometimes the "dictionary form" we find includes numbers before the actual word,
            // for example for ２１度, 4日 etc.
            // so if we fail to find the word we'll try to remove the numbers and try again...
            let mut dictionary_form_without_numbers = String::new();
            let mut reading_without_numbers = String::new();
            if let Some(segmentation) = &furigana::map(
                &dictionary_form,
                &reading_standard.standardised,
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
                    reading_without_numbers.push_str(furigana.furigana.unwrap_or(furigana.segment));
                }
            }

            tracing::info!(
                "trying without numbers {} {} {}",
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
            let counters_char = &['目', '間'];
            let counters = &["目", "間"];
            if !dictionary_form.ends_with(counters_char) {
                return None;
            }
            // same as above but without the counter at the end to account for 人目 and　年間 etc...
            let mut dictionary_form_without_numbers = String::new();
            let mut reading_without_numbers = String::new();
            if let Some(segmentation) = furigana::map(
                &dictionary_form,
                &reading_standard.standardised,
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
                    .take_while(|f| !counters.contains(&f.segment))
                {
                    dictionary_form_without_numbers.push_str(furigana.segment);
                    reading_without_numbers.push_str(furigana.furigana.unwrap_or(furigana.segment));
                }
            }

            tracing::info!(
                "trying trying without numbers and counters {} {} {}",
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

#[cfg(test)]
mod test {
    use super::*;
    use ichiran::IchiranCli;
    use tracing::Level;

    #[test]
    fn here() {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .init();
        let txt = "しつこく、会う";

        let ichiran = IchiranCli::new("../../data/ichiran-cli".into());
        let segments = ichiran.segment(txt, Some(1)).unwrap();
        let segs = to_lbr_segments(
            txt,
            segments,
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        panic!("ohh ye {segs:#?}");
    }
}
