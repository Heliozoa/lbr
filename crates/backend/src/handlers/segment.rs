//! /segment

use crate::{
    authentication::Authentication,
    domain::sentences,
    eq,
    error::{EyreResult, LbrResult},
    query, LbrState,
};
use axum::{extract::State, Json};
use diesel::prelude::*;
use lbr::sentence_splitter::SentenceSplitter;
use lbr_api::{request as req, response as res};

pub async fn segment(
    State(state): State<LbrState>,
    user: Authentication,
    paragraph: Json<req::Paragraph<'static>>,
) -> LbrResult<Json<Vec<res::SegmentedSentence>>> {
    use crate::schema::{sentences as se, sources as so};
    let user_id = user.user_id;
    let req::Paragraph {
        source_id,
        paragraph,
    } = paragraph.0;

    tracing::info!("Segmenting paragraph for source {source_id} and user {user_id}");

    let segmented_sentences = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let _source = so::table
            .filter(so::id.eq(source_id).and(so::user_id.eq(user_id)))
            .select(so::id)
            .get_result::<i32>(&mut conn)?;

        let ignored_word_ids = query::ignored_words(&mut conn, user_id)?;

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
            let segmented_sentence = sentences::process_sentence(
                &state.ichiran_cli,
                sentence.to_string(),
                &ignored_word_ids,
            )?;
            segmented_sentences.push(segmented_sentence);
        }
        EyreResult::Ok(segmented_sentences)
    })
    .await??;
    Ok(Json(segmented_sentences))
}
