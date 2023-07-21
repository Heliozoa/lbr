//! /segment

use super::prelude::*;
use crate::{domain::sentences, queries};
use lbr::sentence_splitter::SentenceSplitter;

#[instrument]
pub async fn segment(
    State(state): State<LbrState>,
    user: Authentication,
    paragraph: Json<req::Paragraph<'static>>,
) -> LbrResult<Json<Vec<res::SegmentedSentence>>> {
    use schema::{sentences as se, sources as so};
    let user_id = user.user_id;
    let req::Paragraph {
        source_id,
        paragraph,
    } = paragraph.0;

    let segmented_sentences = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let _source = so::table
            .filter(so::id.eq(source_id).and(so::user_id.eq(user_id)))
            .select(so::id)
            .get_result::<i32>(&mut conn)?;

        let ignored_word_ids = queries::ignored_words(&mut conn, user_id)?;

        let segmented_sentences = std::thread::scope(|scope| {
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
                    sentences::process_sentence(
                        &state.ichiran_cli,
                        sentence.to_string(),
                        &ignored_word_ids,
                        &state.ichiran_seq_to_word_id,
                    )
                });
                handles.push(segmented_sentence);
            }
            for handle in handles {
                let segmented_sentence = handle.join().expect("Failed to join thread handle")?;
                segmented_sentences.push(segmented_sentence)
            }
            EyreResult::Ok(segmented_sentences)
        })?;
        EyreResult::Ok(segmented_sentences)
    })
    .await??;
    Ok(Json(segmented_sentences))
}
