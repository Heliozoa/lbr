//! /words

use super::prelude::*;
use std::collections::HashMap;

// handlers

#[instrument]
pub async fn ignored_words(
    State(state): State<LbrState>,
    user: Authentication,
) -> LbrResult<Json<Vec<res::IgnoredWord>>> {
    use schema::{ignored_words as iw, word_readings as wr, words as w};

    let ignored_words = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let user_ignored_words = iw::table.filter(iw::user_id.eq(user.user_id));
        let ignored_word_translations = user_ignored_words
            .inner_join(wr::table.on(wr::word_id.eq(iw::word_id)))
            .select(IgnoredWordTranslations::as_select())
            .get_results(&mut conn)?;
        let ignored_word_written_forms = user_ignored_words
            .inner_join(w::table.on(w::id.eq(iw::word_id)))
            .select(IgnoredWordWrittenForm::as_select())
            .get_results(&mut conn)?;
        let ignored_word_readings = user_ignored_words
            .inner_join(wr::table.on(wr::word_id.eq(iw::word_id)))
            .select(IgnoredWordReading::as_select())
            .get_results(&mut conn)?;

        let mut word_id_to_ignored_word = ignored_word_translations
            .into_iter()
            .map(|iwt| {
                (
                    iwt.word_id,
                    res::IgnoredWord {
                        word_id: iwt.word_id,
                        translations: iwt.translations.into_iter().flatten().collect(),
                        written_forms: Vec::new(),
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        let word_id_to_readings = ignored_word_readings
            .into_iter()
            .map(|iwr| (iwr.word_id, iwr.reading))
            .collect::<HashMap<_, _>>();
        for iwwf in ignored_word_written_forms {
            if let Some(iw) = word_id_to_ignored_word.get_mut(&iwwf.word_id) {
                let readings = word_id_to_readings
                    .iter()
                    .filter(|(k, _)| **k == iwwf.word_id)
                    .map(|(_, v)| v.clone())
                    .collect::<Vec<_>>();
                iw.written_forms.push(res::IgnoredWordWrittenForm {
                    written_form: iwwf.word,
                    readings,
                })
            }
        }

        let ignored_words: Vec<res::IgnoredWord> = word_id_to_ignored_word.into_values().collect();
        LbrResult::Ok(ignored_words)
    })
    .await??;

    Ok(Json(ignored_words))
}

#[instrument]
pub async fn delete_ignored_word(
    State(state): State<LbrState>,
    Path(word_id): Path<i32>,
    user: Authentication,
) -> LbrResult<()> {
    use schema::ignored_words as iw;

    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        conn.transaction(|conn| {
            diesel::delete(
                iw::table.filter(iw::user_id.eq(user.user_id).and(iw::word_id.eq(word_id))),
            )
            .execute(conn)?;
            LbrResult::Ok(())
        })?;
        LbrResult::Ok(())
    })
    .await??;

    Ok(())
}

// queries

query! {
    #[derive(Debug)]
    struct IgnoredWordTranslations {
        word_id: i32 = ignored_words::word_id,
        translations: Vec<Option<String>> = word_readings::translations,
    }
}

query! {
    #[derive(Debug)]
    struct IgnoredWordWrittenForm {
        word_id: i32 = ignored_words::word_id,
        word: String = words::word,
    }
}

query! {
    #[derive(Debug)]
    struct IgnoredWordReading {
        word_id: i32 = ignored_words::word_id,
        reading: String = word_readings::reading,
    }
}
