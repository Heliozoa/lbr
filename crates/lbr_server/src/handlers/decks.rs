//! /decks

use super::prelude::*;
use crate::{domain::decks, utils::database::DeckSourceKind};
use std::io::Cursor;

// handlers

#[instrument]
pub async fn get_all(
    State(state): State<LbrState>,
    user: Authentication,
) -> LbrResult<Json<Vec<res::Deck>>> {
    use schema::decks as d;

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

#[instrument]
pub async fn get_one(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
) -> LbrResult<Json<res::DeckDetails>> {
    use schema::{deck_sources as ds, decks as d};

    let user_id = user.user_id;
    let (deck, sources) = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let deck = d::table
            .select(Deck::as_select())
            .filter(d::id.eq(id).and(d::user_id.eq(user_id)))
            .get_result(&mut conn)?;
        let sources = ds::table
            .filter(ds::deck_id.eq(id))
            .select(DeckSource::as_select())
            .load(&mut conn)?;
        EyreResult::Ok((deck, sources))
    })
    .await??;

    let sources = sources
        .into_iter()
        .map(|s| res::DeckSource {
            id: s.id,
            threshold: s.threshold,
            kind: match s.kind {
                DeckSourceKind::Word => res::DeckSourceKind::Word,
                DeckSourceKind::Kanji => res::DeckSourceKind::Kanji,
            },
        })
        .collect();
    let deck = res::DeckDetails {
        id: deck.id,
        name: deck.name,
        sources,
    };
    Ok(Json(deck))
}

#[instrument]
pub async fn insert(
    State(state): State<LbrState>,
    user: Authentication,
    Json(new_deck): Json<req::NewDeck<'static>>,
) -> LbrResult<String> {
    use schema::decks as d;

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

#[instrument]
pub async fn update(
    State(state): State<LbrState>,
    Path(id): Path<i32>,
    user: Authentication,
    Json(update_deck): Json<req::UpdateDeck<'static>>,
) -> LbrResult<()> {
    use schema::{deck_sources as ds, decks as d};

    let user_id = user.user_id;
    let req::UpdateDeck {
        name,
        included_sources,
    } = update_deck;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let decks = d::table
            .filter(d::id.eq(id).and(d::user_id.eq(user_id)))
            .select(d::id)
            .execute(&mut conn)?;
        if decks != 1 {
            return Err(eyre::eyre!("No such deck"));
        }

        conn.transaction(|conn| {
            diesel::update(d::table.filter(d::id.eq(id).and(d::user_id.eq(user_id))))
                .set(eq!(d, name))
                .execute(conn)?;
            diesel::delete(ds::table.filter(ds::deck_id.eq(id))).execute(conn)?;
            let values = included_sources
                .iter()
                .map(|is| {
                    let kind = match is.kind {
                        req::IncludedSourceKind::Kanji => DeckSourceKind::Kanji,
                        req::IncludedSourceKind::Word => DeckSourceKind::Word,
                    };
                    (
                        ds::deck_id.eq(id),
                        ds::source_id.eq(is.source_id),
                        ds::threshold.eq(is.threshold),
                        ds::kind.eq(kind),
                    )
                })
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

#[instrument]
pub async fn delete(
    State(state): State<LbrState>,
    Path(deck_id): Path<i32>,
    user: Authentication,
) -> LbrResult<()> {
    use schema::{deck_sources as ds, decks as d};

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

#[instrument]
pub async fn generate(
    State(state): State<LbrState>,
    Path((id, _filename)): Path<(i32, String)>,
    user: Authentication,
) -> LbrResult<Vec<u8>> {
    use schema::decks as d;

    let user_id = user.user_id;
    let deck_data = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;

        let deck = d::table
            .select(AnkiDeck::as_select())
            .filter(d::id.eq(id).and(d::user_id.eq(user_id)))
            .get_result(&mut conn)?;

        let deck = decks::gen_deck(
            &mut conn,
            deck.name.clone(),
            deck.id,
            deck.anki_deck_id,
            user_id,
        )?;
        let mut buf = Cursor::new(Vec::new());
        deck.write(&mut buf)?;
        EyreResult::Ok(buf)
    })
    .await??;

    Ok(deck_data.into_inner())
}

// queries

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

query! {
    struct AnkiDeck {
        id: i32 = decks::id,
        name: String = decks::name,
        anki_deck_id: i64 = decks::anki_deck_id,
    }
}

query! {
    struct DeckSource {
        id: i32 = deck_sources::source_id,
        threshold: i32 = deck_sources::threshold,
        kind: DeckSourceKind = deck_sources::kind,
    }
}
