//! Functions and types related to sentences.

use super::japanese;
use crate::{eq, error::EyreResult, utils::diesel::PostgresChunks};
use diesel::prelude::*;
use eyre::WrapErr;
use ichiran::{IchiranCli, IchiranError};
use lbr_api::{
    request as req,
    response::{self as res, ApiInterpretation, ApiSegment},
};
use std::collections::{HashMap, HashSet};

/// Segments a sentence using ichiran.
pub fn segment_sentence(
    conn: &mut PgConnection,
    ichiran: &IchiranCli,
    sentence: &str,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
    kanji_to_readings: &HashMap<String, Vec<String>>,
    word_to_meanings: &HashMap<i32, Vec<String>>,
) -> eyre::Result<Vec<ApiSegment>> {
    use crate::schema::words as w;

    tracing::info!("Segmenting sentence '{sentence}'");

    // get individual words from sentence with ichiran
    let segments = match ichiran.segment(sentence, Some(4)) {
        Ok(segments) => segments,
        Err(err) => {
            if let IchiranError::IchiranError { stdout, stderr } = &err {
                tracing::error!("Ichiran error:\n    stdout:\n{stdout}\n    stderr:\n{stderr}");
            }
            return Err(err).wrap_err_with(|| format!("Failed to segment sentence '{sentence}'"));
        }
    };
    let segmented_sentence = lbr::core::to_lbr_segments(
        sentence,
        segments,
        ichiran_word_to_id,
        kanji_to_readings,
        word_to_meanings,
    );

    let mut api_segmented_sentence = Vec::new();
    // convert to database words where applicable
    for segment in segmented_sentence.into_iter() {
        let mut api_interpretations = Vec::new();
        for interpretation in segment.interpretations.into_iter() {
            if let Some(word_id) = interpretation.word_id {
                let (word, reading) = w::table
                    .filter(w::id.eq(word_id))
                    .select((w::word, w::reading_standard))
                    .get_result::<(String, String)>(conn)?;
                api_interpretations.push(ApiInterpretation {
                    word_id: interpretation.word_id,
                    score: interpretation.score,
                    text_word: interpretation.word,
                    text_reading_hiragana: interpretation.reading_hiragana,
                    db_word: word,
                    db_reading_hiragana: reading,
                    meanings: interpretation.meanings,
                });
            }
        }
        api_segmented_sentence.push(ApiSegment {
            text: segment.text,
            interpretations: api_interpretations,
            range: segment.range,
        });
    }

    tracing::info!("Finished segmenting sentence '{sentence}'");
    tracing::trace!("Segmented sentence: {api_segmented_sentence:#?}");
    Ok(api_segmented_sentence)
}

/// Processes a sentence into the appropriate response type.
pub fn process_sentence(
    conn: &mut PgConnection,
    ichiran_cli: &IchiranCli,
    sentence: String,
    ichiran_word_to_id: &HashMap<(i32, String, String), i32>,
    kanji_to_readings: &HashMap<String, Vec<String>>,
    word_to_meanings: &HashMap<i32, Vec<String>>,
) -> eyre::Result<res::SegmentedParagraphSentence> {
    let segments = segment_sentence(
        conn,
        ichiran_cli,
        &sentence,
        ichiran_word_to_id,
        kanji_to_readings,
        word_to_meanings,
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
                        || format!("Failed to map furigana to reading for {reading}"),
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

/*
pub struct BetterSegmentationNode {
    pub contents: BetterSegment,
    pub continuations: Vec<BetterSegmentationNode>,
    pub score: i32,
}

pub enum BetterSegment {
    Word {
        id: i32,
        range: RangeInclusive<usize>,
        reading: String,
    },
    Other(String),
    Empty,
}

fn to_better_lbr_segments(mut segments: Vec<ichiran::Segment>) -> BetterSegmentationNode {
    if let Some(segment) = segments.pop() {
        match segment {
            ichiran::Segment::Segmentations(mut segmentations) => {
                // list of alternate segmentations
                for segmentation in segmentations {
                    // words in the segmentation
                    for word in segmentation.words {
                        // alternative interpretations for the word
                        for alternative in word.alternatives {
                            match alternative {
                                ichiran::Alternative::WordInfo(wi) => {
                                    //
                                }
                                ichiran::Alternative::CompoundWordInfo(cwi) => {
                                    // components of compound word
                                    for component in cwi.components {
                                        //
                                    }
                                }
                            }
                        }
                    }
                }
                todo!()
            }
            ichiran::Segment::Other(other) => {
                // text segment, take as content and move on
                BetterSegmentationNode {
                    contents: BetterSegment::Other(other),
                    continuations: to_better_lbr_segments(segments),
                    score: 0,
                }
            }
        }
    } else {
        BetterSegmentationNode {
            contents: BetterSegment::Empty,
            continuations: Vec::new(),
            score: 0,
        }
    }
}

 */
