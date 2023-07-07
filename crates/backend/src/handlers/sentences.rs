//! /sentences

use crate::{
    authentication::Authentication,
    eq,
    error::{EyreResult, LbrResult},
    query,
    utils::{database, diesel::PostgresChunks},
    LbrState,
};
use axum::{extract::Path, Json};
use diesel::prelude::*;
use lbr_api::{request as req, response as res};

query! {
    struct Sentence {
        id: i32 = sentences::id,
        sentence: String = sentences::sentence,
    }
}

pub async fn get(state: LbrState) -> LbrResult<Json<Vec<res::Sentence>>> {
    use crate::schema::sentences as s;
    tracing::info!("Fetching sentences");

    let sentences = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let sentences = s::table.select(Sentence::as_select()).load(&mut conn)?;
        EyreResult::Ok(sentences)
    })
    .await??
    .into_iter()
    .map(|s| res::Sentence {
        id: s.id,
        sentence: s.sentence,
    })
    .collect();

    Ok(Json(sentences))
}

pub async fn insert(
    state: LbrState,
    user: Authentication,
    new_sentence: Json<req::NewSentence<'static>>,
) -> LbrResult<()> {
    use crate::schema::{sentence_words as sw, sentences as s};
    tracing::info!("Inserting sentence");

    let req::NewSentence {
        source_id,
        deck_id: _,
        sentence,
        sentence_words,
    } = new_sentence.0;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        // TODO: check if identical sentence already exists in db
        conn.transaction(|conn| {
            let sentence_id = diesel::insert_into(s::table)
                .values(eq!(s, sentence, source_id))
                .returning(s::id)
                .get_result::<i32>(conn)?;
            let sw_values = sentence_words
                .into_iter()
                .map(
                    |req::NewSentenceWord {
                         word_id,
                         reading,
                         idx_start,
                         idx_end,
                         furigana,
                     }| {
                        let furigana = furigana
                            .into_iter()
                            .map(|f| database::Furigana {
                                word_start_idx: f.word_start_idx,
                                word_end_idx: f.word_end_idx,
                                reading_start_idx: f.reading_start_idx,
                                reading_end_idx: f.reading_end_idx,
                            })
                            .collect::<Vec<_>>();
                        eq!(
                            sw,
                            sentence_id,
                            word_id,
                            reading,
                            idx_start,
                            idx_end,
                            furigana
                        )
                    },
                )
                .collect::<Vec<_>>();
            for chunk in sw_values.pg_chunks() {
                diesel::insert_into(sw::table).values(chunk).execute(conn)?;
            }
            EyreResult::Ok(())
        })?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

pub async fn get_one(
    state: LbrState,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<Json<res::Sentence>> {
    todo!()
}

pub async fn update(
    state: LbrState,
    Path(id): Path<i32>,
    user: Authentication,
    update_sentence: Json<req::UpdateSentence<'static>>,
) -> LbrResult<()> {
    use crate::schema::{sentence_words as sw, sentences as s};
    tracing::info!("Updating sentence {id}");

    let req::UpdateSentence {
        sentence,
        sentence_words,
    } = update_sentence.0;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        conn.transaction(|conn| {
            diesel::update(s::table.filter(eq!(s, id)))
                .set(eq!(s, sentence))
                .execute(conn)?;
            let sentence_id = id;
            diesel::delete(sw::table.filter(eq!(sw, sentence_id))).execute(conn)?;
            let sw_values = sentence_words
                .iter()
                .map(
                    |req::UpdatedSentenceWord {
                         word_id,
                         reading,
                         idx_start,
                         idx_end,
                         furigana,
                     }| {
                        let furigana = furigana
                            .iter()
                            .map(|f| database::Furigana {
                                word_start_idx: f.word_start_idx,
                                word_end_idx: f.word_end_idx,
                                reading_start_idx: f.reading_start_idx,
                                reading_end_idx: f.reading_end_idx,
                            })
                            .collect::<Vec<_>>();
                        eq!(
                            sw,
                            sentence_id,
                            word_id,
                            reading,
                            idx_start,
                            idx_end,
                            furigana
                        )
                    },
                )
                .collect::<Vec<_>>();
            for chunk in sw_values.pg_chunks() {
                diesel::insert_into(sw::table).values(chunk).execute(conn)?;
            }
            EyreResult::Ok(())
        })?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

pub async fn delete(state: LbrState, Path(id): Path<i32>, user: Authentication) -> LbrResult<()> {
    use crate::schema::{sentence_words as sw, sentences as s};
    tracing::info!("Deleting sentence {id}");

    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        conn.transaction(|conn| {
            diesel::delete(sw::table.filter(sw::sentence_id.eq(id))).execute(conn)?;
            diesel::delete(s::table.filter(s::id.eq(id))).execute(conn)?;
            EyreResult::Ok(())
        })?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}
