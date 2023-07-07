//! Functions and types related to sentences.

use eyre::WrapErr;
use ichiran::IchiranCli;
use lbr_core::ichiran_types;

/// Segments a sentence using ichiran.
pub fn process(ichiran: &IchiranCli, sentence: &str) -> eyre::Result<Vec<ichiran_types::Segment>> {
    // get individual words from sentence with ichiran
    let segments = ichiran
        .segment(sentence)
        .wrap_err_with(|| format!("Failed to segment sentence '{sentence}'"))?;
    let segmented_sentence = lbr::core::to_lbr_segments(sentence, segments);
    Ok(segmented_sentence)
}
