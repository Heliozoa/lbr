//! Functions and types related to sentences.

use eyre::WrapErr;
use ichiran::IchiranCli;
use lbr_api::response as res;
use lbr_core::ichiran_types;
use std::collections::HashSet;

/// Segments a sentence using ichiran.
pub fn segment_sentence(
    ichiran: &IchiranCli,
    sentence: &str,
) -> eyre::Result<Vec<ichiran_types::Segment>> {
    // get individual words from sentence with ichiran
    let segments = ichiran
        .segment(sentence)
        .wrap_err_with(|| format!("Failed to segment sentence '{sentence}'"))?;
    let segmented_sentence = lbr::core::to_lbr_segments(sentence, segments);
    Ok(segmented_sentence)
}

/// Processes a sentence into the appropriate response type.
pub fn process_sentence(
    ichiran_cli: &IchiranCli,
    sentence: String,
    ignored_word_ids: &HashSet<i32>,
) -> eyre::Result<res::SegmentedSentence> {
    let segments = segment_sentence(&ichiran_cli, &sentence)?;
    let segment_word_ids = segments
        .iter()
        .filter_map(|s| {
            if let ichiran_types::Segment::Phrase {
                interpretations, ..
            } = s
            {
                Some(interpretations)
            } else {
                None
            }
        })
        .flatten()
        .flat_map(|i| &i.components)
        .filter_map(|c| c.word_id)
        .collect::<Vec<_>>();
    let ignored_words = segment_word_ids
        .iter()
        .copied()
        .filter(|swi| ignored_word_ids.contains(&swi))
        .collect();
    Ok(res::SegmentedSentence {
        sentence: sentence.to_string(),
        segments,
        ignored_words,
    })
}
