//! /sentences

use super::prelude::*;
use crate::{
    domain::sentences::{self, NewSentenceWords},
    utils::database,
};

// handlers

#[instrument]
pub async fn get_one(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<Json<res::SentenceDetails>> {
    use schema::{
        sentence_words as sw, sentences as s, sources as so, word_readings as wr, words as w,
    };

    let sentence = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;

        let sentence = s::table
            .inner_join(so::table.on(s::source_id.eq(so::id)))
            .filter(so::user_id.eq(user.user_id).and(s::id.eq(id)))
            .select(Sentence::as_select())
            .get_result(&mut conn)?;
        let words = sw::table
            .inner_join(w::table.on(sw::word_id.eq(w::id.nullable())))
            .inner_join(wr::table.on(wr::word_id.eq(w::id)))
            .filter(sw::sentence_id.eq(id))
            .select(SentenceWord::as_select())
            .load(&mut conn)?;
        let words = words
            .into_iter()
            .map(|sw| res::SentenceWord {
                reading: sw.reading,
                idx_start: sw.idx_start,
                idx_end: sw.idx_end,
                furigana: sw
                    .furigana
                    .into_iter()
                    .flatten()
                    .map(|f| res::Furigana {
                        word_start_idx: f.word_start_idx,
                        word_end_idx: f.word_end_idx,
                        reading_start_idx: f.reading_start_idx,
                        reading_end_idx: f.reading_end_idx,
                    })
                    .collect(),
                translations: sw.translations.into_iter().flatten().collect(),
            })
            .collect();
        let sentence = res::SentenceDetails {
            id: sentence.id,
            sentence: sentence.sentence,
            words,
        };
        EyreResult::Ok(sentence)
    })
    .await??;

    Ok(Json(sentence))
}

#[instrument]
pub async fn update(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
    update_sentence: Json<req::SegmentedSentence>,
) -> LbrResult<()> {
    use schema::{sentence_words as sw, sentences as s, sources as so};

    let req::SegmentedSentence {
        sentence,
        words,
        ignore_words,
    } = update_sentence.0;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;

        let sentence = &sentence;
        let sentence_id = s::table
            .inner_join(so::table.on(so::id.eq(s::source_id)))
            .filter(s::id.eq(id).and(so::user_id.eq(user.user_id)))
            .select(s::id)
            .get_result::<i32>(&mut conn)?;

        conn.transaction(|conn| {
            diesel::update(s::table.filter(eq!(s, id)))
                .set(eq!(s, sentence))
                .execute(conn)?;
            diesel::delete(sw::table.filter(eq!(sw, sentence_id))).execute(conn)?;
            sentences::insert_sentence_words(
                conn,
                &state.kanji_to_readings,
                NewSentenceWords {
                    user_id: user.user_id,
                    sentence_id,
                    sentence,
                    words,
                    ignore_words,
                },
            )?;
            EyreResult::Ok(())
        })?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

#[instrument]
pub async fn delete(
    State(state): State<LbrState>,
    Path(sentence_id): Path<i32>,
    user: Authentication,
) -> LbrResult<()> {
    use schema::{sentence_words as sw, sentences as s, sources as so};

    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let id = s::table
            .inner_join(so::table.on(so::id.eq(s::source_id)))
            .filter(so::user_id.eq(user.user_id).and(s::id.eq(sentence_id)))
            .select(s::id)
            .get_result::<i32>(&mut conn)?;
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

#[instrument]
pub async fn segment(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<Json<res::SegmentedSentence>> {
    use schema::sentences as s;

    let segmented_sentence = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let sentence = s::table
            .filter(s::id.eq(id))
            .select(s::sentence)
            .get_result::<String>(&mut conn)?;
        let segmented_sentence = sentences::process_sentence(
            &state.ichiran_cli,
            sentence,
            &state.ichiran_seq_to_word_id,
        )?;
        EyreResult::Ok(segmented_sentence)
    })
    .await??;

    Ok(Json(segmented_sentence))
}

// queries

query! {
    struct Sentence {
        id: i32 = sentences::id,
        sentence: String = sentences::sentence,
    }
}

query! {
    struct SentenceWord {
        reading: Option<String> = sentence_words::reading,
        idx_start: i32 = sentence_words::idx_start,
        idx_end: i32 = sentence_words::idx_end,
        furigana: Vec<Option<database::Furigana>> = sentence_words::furigana,
        translations: Vec<Option<String>> = word_readings::translations,
    }
}
