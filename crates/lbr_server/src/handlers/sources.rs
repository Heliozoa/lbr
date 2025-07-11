//! /sources

use super::prelude::*;
use crate::domain::sentences::{self, NewSentenceWords};

// handlers

#[instrument]
pub async fn get_all(
    State(state): State<LbrState>,
    user: Authentication,
) -> LbrResult<Json<Vec<res::Source>>> {
    use schema::sources as s;

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

#[instrument]
pub async fn get_one(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<Json<res::Source>> {
    use schema::sources as so;

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

#[instrument]
pub async fn get_details(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<Json<res::SourceDetails>> {
    use schema::{sentences as se, sources as so};

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
            .order_by(se::id)
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

#[instrument]
pub async fn insert(
    State(state): State<LbrState>,
    user: Authentication,
    new_source: Json<req::NewSource<'static>>,
) -> LbrResult<String> {
    use schema::sources as s;

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

#[instrument]
pub async fn update(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
    update_source: Json<req::UpdateSource<'static>>,
) -> LbrResult<()> {
    use schema::sources as s;

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

#[instrument]
pub async fn delete(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<()> {
    use schema::{deck_sources as ds, sentence_words as sw, sentences as se, sources as so};

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

#[instrument]
pub async fn add_sentence(
    State(state): State<LbrState>,
    Path(source_id): Path<i32>,
    user: Authentication,
    sentence: Json<req::SegmentedSentence>,
) -> LbrResult<()> {
    use schema::{sentences as se, sources as so};

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
            .filter(so::id.eq(source_id).and(so::user_id.eq(user_id)))
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

// queries

query! {
    struct Sentence {
        id: i32 = sentences::id,
        sentence: String = sentences::sentence,
    }
}

query! {
    struct Source {
        id: i32 = sources::id,
        name: String = sources::name,
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
