//! /segment

use super::prelude::*;
use crate::{domain::sentences, queries};
use lbr::sentence_splitter::SentenceSplitter;
use std::collections::HashSet;

#[instrument]
pub async fn segment(
    State(state): State<LbrState>,
    user: Authentication,
    paragraph: Json<req::Paragraph<'static>>,
) -> LbrResult<Json<res::SegmentedParagraph>> {
    use schema::{sentences as se, sources as so};
    let user_id = user.user_id;
    let req::Paragraph {
        source_id,
        paragraph,
    } = paragraph.0;

    let segmented_paragraph = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let source_id = so::table
            .filter(so::id.eq(source_id).and(so::user_id.eq(user_id)))
            .select(so::id)
            .get_result::<i32>(&mut conn)?;

        let ignored_word_ids = queries::ignored_words(&mut conn, user_id)?;

        let segmented_paragraph = std::thread::scope(|scope| {
            let mut segmented_sentences = Vec::new();
            let mut handles = Vec::new();
            for sentence in SentenceSplitter::new(&paragraph) {
                let existing_sentences = se::table
                    .select(se::id)
                    .filter(eq!(se, sentence).and(eq!(se, source_id)))
                    .execute(&mut conn)?;
                if existing_sentences != 0 {
                    tracing::info!("Skipping existing sentence {sentence}");
                    continue;
                }
                let segmented_sentence = scope.spawn(|| {
                    let mut conn = state.lbr_pool.get()?;
                    sentences::process_sentence(
                        &mut conn,
                        &state.ichiran_cli,
                        sentence.to_string(),
                        &state.ichiran_word_to_id,
                        &state.kanji_to_readings,
                        &state.word_to_meanings,
                    )
                });
                handles.push(segmented_sentence);
            }
            let mut word_ids = HashSet::new();
            for handle in handles {
                let segmented_sentence = handle.join().expect("Failed to join thread handle")?;
                for segment in &segmented_sentence.segments {
                    for interpretation in &segment.interpretations {
                        if let Some(word_id) = interpretation.word_id {
                            word_ids.insert(word_id);
                        }
                    }
                }
                segmented_sentences.push(segmented_sentence)
            }
            EyreResult::Ok(res::SegmentedParagraph {
                sentences: segmented_sentences,
                ignored_words: word_ids.intersection(&ignored_word_ids).copied().collect(),
            })
        })?;
        EyreResult::Ok(segmented_paragraph)
    })
    .await??;
    Ok(Json(segmented_paragraph))
}
