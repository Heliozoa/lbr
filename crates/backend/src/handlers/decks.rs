//! /decks

use crate::{
    authentication::Authentication,
    domain::decks,
    eq,
    error::{EyreResult, LbrResult},
    query,
    utils::diesel::PostgresChunks,
    LbrState,
};
use axum::{extract::Path, Json};
use diesel::prelude::*;
use lbr_api::{request as req, response as res};
use std::io::Read;

query! {
    struct Deck {
        pub id: i32 = decks::id,
        pub name: String = decks::name,
    }
}

impl From<Deck> for res::Deck {
    fn from(value: Deck) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

/// Returns all decks owned by the user
pub async fn get_all(state: LbrState, user: Authentication) -> LbrResult<Json<Vec<res::Deck>>> {
    use crate::schema::decks as d;
    tracing::info!("Fetching decks");

    let user_id = user.user_id;
    let decks = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let decks = d::table
            .select(Deck::as_select())
            .filter(eq!(d, user_id))
            .get_results(&mut conn)?;
        EyreResult::Ok(decks)
    })
    .await??
    .into_iter()
    .map(Into::into)
    .collect();

    Ok(Json(decks))
}

/// Returns the deck with the given id and owner
pub async fn get_one(
    state: LbrState,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<Json<res::DeckDetails>> {
    use crate::schema::{deck_sources as ds, decks as d};
    tracing::info!("Fetching decks");

    let user_id = user.user_id;
    let (deck, sources) = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let deck = d::table
            .select(Deck::as_select())
            .filter(d::id.eq(id).and(d::user_id.eq(user_id)))
            .get_result(&mut conn)?;
        let sources = ds::table
            .select(ds::source_id)
            .get_results::<i32>(&mut conn)?;
        EyreResult::Ok((deck, sources))
    })
    .await??;

    let deck = res::DeckDetails {
        id: deck.id,
        name: deck.name,
        sources,
    };
    Ok(Json(deck))
}

/// Inserts a new deck for the user
pub async fn insert(
    state: LbrState,
    user: Authentication,
    Json(new_deck): Json<req::NewDeck<'static>>,
) -> LbrResult<String> {
    use crate::schema::decks as d;
    tracing::info!("Inserting deck");

    let user_id = user.user_id;
    let req::NewDeck { name } = new_deck;
    let anki_deck_id = rand::random::<i64>();
    let id = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let id = diesel::insert_into(d::table)
            .values(eq!(d, anki_deck_id, name, user_id))
            .returning(d::id)
            .get_result::<i32>(&mut conn)?;
        EyreResult::Ok(id)
    })
    .await??;

    Ok(id.to_string())
}

/// Updates the deck with the given id and owner
pub async fn update(
    state: LbrState,
    user: Authentication,
    Path(id): Path<i32>,
    Json(update_deck): Json<req::UpdateDeck<'static>>,
) -> LbrResult<()> {
    use crate::schema::decks as d;
    tracing::info!("Updating deck {id}");

    let user_id = user.user_id;
    let req::UpdateDeck { name } = update_deck;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        conn.transaction(|conn| {
            diesel::update(d::table.filter(d::id.eq(id).and(d::user_id.eq(user_id))))
                .set(eq!(d, name))
                .execute(conn)?;
            EyreResult::Ok(())
        })?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

/// Deletes the deck with the given id and owner
pub async fn delete(
    state: LbrState,
    user: Authentication,
    Path(deck_id): Path<i32>,
) -> LbrResult<()> {
    use crate::schema::{deck_sources as ds, decks as d};
    tracing::info!("Deleting deck {deck_id}");

    let user_id = user.user_id;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        conn.transaction(|conn| {
            diesel::delete(ds::table.filter(eq!(ds, deck_id))).execute(conn)?;
            let id = deck_id;
            let decks_deleted =
                diesel::delete(d::table.filter(d::id.eq(id).and(d::user_id.eq(user_id))))
                    .execute(conn)?;
            if decks_deleted != 1 {
                return Err(eyre::eyre!("No such deck"));
            }
            EyreResult::Ok(())
        })?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

pub async fn update_sources(
    state: LbrState,
    user: Authentication,
    Path(id): Path<i32>,
    Json(sources): Json<req::UpdateDeckSources<'static>>,
) -> LbrResult<()> {
    use crate::schema::{deck_sources as ds, decks as d};

    let user_id = user.user_id;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let deck = d::table.select(eq!(d, id, user_id)).execute(&mut conn)?;
        if deck != 1 {
            return Err(eyre::eyre!("No such deck"));
        }

        conn.transaction(move |conn| {
            diesel::delete(ds::table.filter(ds::deck_id.eq(id))).execute(conn)?;
            let values = sources
                .included_sources
                .into_iter()
                .map(|source_id| (ds::deck_id.eq(id), ds::source_id.eq(source_id)))
                .collect::<Vec<_>>();
            for chunk in values.pg_chunks() {
                diesel::insert_into(ds::table).values(chunk).execute(conn)?;
            }
            EyreResult::Ok(())
        })?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

query! {
    struct AnkiDeck {
        id: i32 = decks::id,
        name: String = decks::name,
        anki_deck_id: i64 = decks::anki_deck_id,
    }
}

pub async fn generate(
    state: LbrState,
    user: Authentication,
    Path((id, _filename)): Path<(i32, String)>,
) -> LbrResult<Vec<u8>> {
    use crate::schema::decks as d;
    tracing::info!("Generating deck {id}");

    let user_id = user.user_id;
    let deck_data = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;

        let deck = d::table
            .select(AnkiDeck::as_select())
            .filter(d::id.eq(id).and(d::user_id.eq(user_id)))
            .get_result(&mut conn)?;

        let mut temp = tempfile::NamedTempFile::new()?;
        let temp_path = temp
            .path()
            .as_os_str()
            .to_str()
            .ok_or_else(|| eyre::eyre!("Invalid temporary path"))?;
        let deck = decks::gen_deck(&mut conn, &deck.name, deck.id, deck.anki_deck_id)?;
        deck.write_to_file(temp_path)?;
        let mut buf = Vec::new();
        temp.read_to_end(&mut buf)?;
        EyreResult::Ok(buf)
    })
    .await??;

    Ok(deck_data)
}
