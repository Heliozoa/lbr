//! Functions and types related to sentences.

use super::japanese;
use crate::{eq, error::EyreResult, utils::diesel::PostgresChunks};
use diesel::prelude::*;
use eyre::WrapErr;
use ichiran::{IchiranCli, IchiranError};
use lbr_api::{request as req, response as res};
use lbr_core::ichiran_types;
use std::collections::{HashMap, HashSet};

/// Segments a sentence using ichiran.
pub fn segment_sentence(
    ichiran: &IchiranCli,
    sentence: &str,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
    kanji_to_readings: &HashMap<String, Vec<String>>,
) -> eyre::Result<Vec<ichiran_types::Segment>> {
    tracing::info!("Segmenting sentence '{sentence}'");

    // get individual words from sentence with ichiran
    let segments = match ichiran.segment(sentence) {
        Ok(segments) => segments,
        Err(err) => {
            if let IchiranError::IchiranError { stdout, stderr } = &err {
                tracing::error!("Ichiran error:\n    stdout:\n{stdout}\n    stderr:\n{stderr}");
            }
            return Err(err).wrap_err_with(|| format!("Failed to segment sentence '{sentence}'"));
        }
    };
    let segmented_sentence =
        lbr::core::to_lbr_segments(sentence, segments, ichiran_word_to_id, kanji_to_readings);

    tracing::info!("Finished segmenting sentence '{sentence}': {segmented_sentence:#?}");
    Ok(segmented_sentence)
}

/// Processes a sentence into the appropriate response type.
pub fn process_sentence(
    ichiran_cli: &IchiranCli,
    sentence: String,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
    kanji_to_readings: &HashMap<String, Vec<String>>,
) -> eyre::Result<res::SegmentedParagraphSentence> {
    let segments = segment_sentence(
        ichiran_cli,
        &sentence,
        ichiran_word_to_id,
        kanji_to_readings,
    )?;
    Ok(res::SegmentedParagraphSentence { sentence, segments })
}

pub struct NewSentenceWords<'a> {
    pub user_id: i32,
    pub sentence_id: i32,
    pub sentence: &'a str,
    pub words: Vec<req::Word>,
    pub ignore_words: HashSet<i32>,
}

pub fn insert_sentence_words(
    conn: &mut PgConnection,
    kanji_to_readings: &HashMap<String, Vec<String>>,
    new_sentence_words: NewSentenceWords<'_>,
) -> eyre::Result<()> {
    use crate::schema::{ignored_words as iw, sentence_words as sw};

    let NewSentenceWords {
        user_id,
        sentence_id,
        sentence,
        words,
        ignore_words,
    } = new_sentence_words;

    conn.transaction(move |conn| {
        let mut sentence_words = Vec::new();
        for req::Word {
            id: word_id,
            reading,
            idx_start,
            idx_end,
        } in words
        {
            let word = sentence
                .get(idx_start as usize..idx_end as usize)
                .ok_or_else(|| eyre::eyre!("Request had invalid indexes for word"))?;
            let furigana = reading
                .as_ref()
                .map(|reading| {
                    japanese::map_to_db_furigana(word, reading, kanji_to_readings).wrap_err_with(
                        || format!("Failed to map furigana to reading for {}", reading),
                    )
                })
                .transpose()?
                .unwrap_or_default();
            sentence_words.push(eq!(
                sw,
                sentence_id,
                word_id,
                reading,
                idx_start,
                idx_end,
                furigana
            ));
        }
        for chunk in sentence_words.pg_chunks() {
            diesel::insert_into(sw::table)
                .values(chunk)
                .execute(conn)
                .wrap_err("Failed to insert sentence word")?;
        }
        let ignored_words = ignore_words
            .into_iter()
            .map(|word_id| eq!(iw, word_id, user_id))
            .collect::<Vec<_>>();
        for chunk in ignored_words.pg_chunks() {
            diesel::insert_into(iw::table)
                .values(chunk)
                .on_conflict((iw::word_id, iw::user_id))
                .do_nothing()
                .execute(conn)
                .wrap_err("Failed to insert ignored words")?;
        }
        EyreResult::Ok(())
    })?;
    Ok(())
}
