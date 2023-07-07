//! /segment

use crate::{
    authentication::Authentication,
    domain::sentences,
    eq,
    error::{EyreResult, LbrResult},
    LbrState,
};
use axum::Json;
use diesel::prelude::*;
use lbr::sentence_splitter::SentenceSplitter;
use lbr_api::{request as req, response as res};
use lbr_core::ichiran_types::Segment;
use std::collections::HashSet;

pub async fn segment(
    state: LbrState,
    user: Authentication,
    paragraph: Json<req::Paragraph<'static>>,
) -> LbrResult<Json<Vec<res::SegmentedSentence>>> {
    use crate::schema::{ignored_words as iw, sentences as se, sources as so};
    tracing::info!("Segmenting paragraph");

    let user_id = user.user_id;
    let req::Paragraph {
        source_id,
        paragraph,
    } = paragraph.0;
    let segmented_sentences = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let source = so::table
            .select(so::id.eq(source_id).and(so::user_id.eq(user_id)))
            .execute(&mut conn)?;
        if source != 1 {
            return Err(eyre::eyre!("No such source"));
        }

        let ignored_word_ids = iw::table
            .select(iw::word_id)
            .filter(iw::user_id.eq(user_id))
            .get_results::<i32>(&mut conn)?
            .into_iter()
            .collect::<HashSet<_>>();

        let mut segmented_sentences = Vec::new();
        for sentence in SentenceSplitter::new(&paragraph) {
            let existing_sentences = se::table
                .select(se::id)
                .filter(eq!(se, sentence).and(eq!(se, source_id)))
                .execute(&mut conn)?;
            if existing_sentences != 0 {
                tracing::info!("Skipping existing sentence {sentence}");
                continue;
            }
            let segments = sentences::process(&state.ichiran_cli, sentence)?;
            let segment_word_ids = segments
                .iter()
                .filter_map(|s| {
                    if let Segment::Phrase {
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
            segmented_sentences.push(res::SegmentedSentence {
                sentence: sentence.to_string(),
                segments,
                ignored_words,
            });
        }
        EyreResult::Ok(segmented_sentences)
    })
    .await??;
    Ok(Json(segmented_sentences))
}
