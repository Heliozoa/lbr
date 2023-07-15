//! /sources

use crate::{
    authentication::Authentication,
    domain::japanese,
    eq,
    error::{EyreResult, LbrResult},
    query,
    utils::diesel::PostgresChunks,
    LbrState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use diesel::prelude::*;
use eyre::Context;
use lbr_api::{request as req, response as res};

query! {
    pub struct Source {
        pub id: i32 = sources::id,
        pub name: String = sources::name,
    }
}

impl From<Source> for res::Source {
    fn from(value: Source) -> Self {
        res::Source {
            id: value.id,
            name: value.name,
        }
    }
}

/// Gets the user's sources
pub async fn get_all(
    State(state): State<LbrState>,
    user: Authentication,
) -> LbrResult<Json<Vec<res::Source>>> {
    use crate::schema::sources as s;
    tracing::info!("Fetching sources");

    let user_id = user.user_id;
    let sources = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let sources = s::table
            .select(Source::as_select())
            .filter(eq!(s, user_id))
            .get_results(&mut conn)?;
        EyreResult::Ok(sources)
    })
    .await??
    .into_iter()
    .map(Into::into)
    .collect();

    Ok(Json(sources))
}

query! {
    struct Sentence {
        id: i32 = sentences::id,
        sentence: String = sentences::sentence,
    }
}

/// Gets the given source for the user
pub async fn get_one(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<Json<res::Source>> {
    use crate::schema::sources as so;
    tracing::info!("Fetching source");

    let user_id = user.user_id;
    let source = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let source = so::table
            .select(Source::as_select())
            .filter(so::id.eq(id).and(so::user_id.eq(user_id)))
            .get_result(&mut conn)?;
        EyreResult::Ok(source)
    })
    .await??;

    Ok(Json(res::Source {
        id: source.id,
        name: source.name,
    }))
}

/// Gets the given source for the user
pub async fn get_details(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<Json<res::SourceDetails>> {
    use crate::schema::{sentences as se, sources as so};
    tracing::info!("Fetching source");

    let user_id = user.user_id;
    let (source, sentences) = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let source = so::table
            .select(Source::as_select())
            .filter(so::id.eq(id).and(so::user_id.eq(user_id)))
            .get_result(&mut conn)?;
        let source_id = source.id;
        let sentences = se::table
            .select(Sentence::as_select())
            .filter(eq!(se, source_id))
            .get_results(&mut conn)?;
        EyreResult::Ok((source, sentences))
    })
    .await??;

    Ok(Json(res::SourceDetails {
        id: source.id,
        name: source.name,
        sentences: sentences
            .into_iter()
            .map(|s| res::Sentence {
                id: s.id,
                sentence: s.sentence,
            })
            .collect(),
    }))
}

/// Inserts a new source for the user
pub async fn insert(
    State(state): State<LbrState>,
    user: Authentication,
    new_source: Json<req::NewSource<'static>>,
) -> LbrResult<String> {
    use crate::schema::sources as s;
    tracing::info!("Inserting source");

    let user_id = user.user_id;
    let req::NewSource { name } = new_source.0;
    let id = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let id = diesel::insert_into(s::table)
            .values(eq!(s, name, user_id))
            .returning(s::id)
            .get_result::<i32>(&mut conn)?;
        EyreResult::Ok(id)
    })
    .await??;

    Ok(id.to_string())
}

pub async fn update(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
    update_source: Json<req::UpdateSource<'static>>,
) -> LbrResult<()> {
    use crate::schema::sources as s;
    tracing::info!("Updating source {id}");

    let user_id = user.user_id;
    let req::UpdateSource { name } = update_source.0;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        diesel::update(s::table.filter(s::id.eq(id).and(s::user_id.eq(user_id))))
            .set(eq!(s, name))
            .execute(&mut conn)?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

pub async fn delete(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<()> {
    use crate::schema::{deck_sources as ds, sentence_words as sw, sentences as se, sources as so};
    tracing::info!("Deleting source {id}");

    let user_id = user.user_id;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        conn.transaction(move |conn| {
            diesel::delete(ds::table.filter(ds::source_id.eq(id))).execute(conn)?;
            let sentence_ids = se::table
                .select(se::id)
                .filter(se::source_id.eq(id))
                .get_results::<i32>(conn)?;
            diesel::delete(sw::table.filter(sw::sentence_id.eq_any(sentence_ids))).execute(conn)?;
            diesel::delete(se::table.filter(se::source_id.eq(id))).execute(conn)?;
            diesel::delete(so::table.filter(so::id.eq(id).and(so::user_id.eq(user_id))))
                .execute(conn)?;
            EyreResult::Ok(())
        })?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

pub async fn add_sentence(
    State(state): State<LbrState>,
    Path(source_id): Path<i32>,
    user: Authentication,
    sentence: Json<req::SegmentedSentence>,
) -> LbrResult<()> {
    use crate::schema::{
        ignored_words as iw, sentence_words as sw, sentences as se, sources as so,
    };
    tracing::info!("Adding sentence to source {source_id}");

    let user_id = user.user_id;
    let req::SegmentedSentence {
        sentence,
        words,
        ignore_words,
    } = sentence.0;
    tokio::task::spawn_blocking(move || {
        let sentence = &sentence;
        let mut conn = state.lbr_pool.get().wrap_err("Failed to get pool")?;
        let sources = so::table
            .select(so::id)
            .filter(eq!(so, user_id))
            .execute(&mut conn)
            .wrap_err("Failed to execute query to fetch source for user")?;
        if sources != 1 {
            return Err(eyre::eyre!("No such source"));
        }
        conn.transaction(move |conn| {
            let sentence_id = diesel::insert_into(se::table)
                .values(eq!(se, sentence, source_id))
                .returning(se::id)
                .get_result::<i32>(conn)
                .wrap_err("Failed to insert sentence")?;
            let ichiran_seq_to_word_id = &state.clone().ichiran_seq_to_word_id;

            let mut sentence_words = Vec::new();
            for req::Word {
                id: ichiran_id,
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
                        japanese::map_to_db_furigana(word, reading, &state.kanji_to_readings)
                            .wrap_err_with(|| {
                                format!("Failed to map furigana to reading for {}", reading)
                            })
                    })
                    .transpose()?
                    .unwrap_or_default();
                let word_id = ichiran_seq_to_word_id
                    .get(&ichiran_id)
                    .copied()
                    .ok_or_else(|| eyre::eyre!("No word found for ichiran seq {ichiran_id}"))?;
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
                    .wrap_err("Failed to insert sentece word")?;
            }
            let ignored_words = ignore_words
                .into_iter()
                .map(|ichiran_seq| {
                    ichiran_seq_to_word_id
                        .get(&ichiran_seq)
                        .copied()
                        .ok_or_else(|| eyre::eyre!("Failed to find word id for {ichiran_seq}"))
                })
                .map(|word_id| word_id.map(|word_id| eq!(iw, word_id, user_id)))
                .collect::<Result<Vec<_>, _>>()?;
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
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}
