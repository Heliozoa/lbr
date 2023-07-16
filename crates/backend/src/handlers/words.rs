use crate::{authentication::Authentication, error::LbrResult, query, LbrState};
use axum::{
    extract::{Path, State},
    Json,
};
use diesel::prelude::*;
use itertools::Itertools;
use lbr_api::response as res;
use std::collections::HashMap;

pub async fn ignored_words(
    State(state): State<LbrState>,
    user: Authentication,
) -> LbrResult<Json<Vec<res::IgnoredWord>>> {
    use crate::schema::{
        ignored_words as iw, word_readings as wr, words as w, written_forms as wf,
    };
    tracing::info!("Fetching ignored words");

    let ignored_words = tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let user_ignored_words = iw::table.filter(iw::user_id.eq(user.user_id));
        let ignored_word_translations = user_ignored_words
            .inner_join(w::table.on(w::id.eq(iw::word_id)))
            .select(IgnoredWordTranslations::as_select())
            .get_results(&mut conn)?;
        let ignored_word_written_forms = user_ignored_words
            .inner_join(wf::table.on(wf::word_id.eq(iw::word_id)))
            .select(IgnoredWordWrittenForm::as_select())
            .get_results(&mut conn)?;
        let ignored_word_readings = user_ignored_words
            .inner_join(wf::table.on(wf::word_id.eq(iw::word_id)))
            .inner_join(wr::table.on(wr::written_form_id.eq(wf::id)))
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
        let mut written_form_id_to_readings = ignored_word_readings
            .into_iter()
            .map(|iwr| (iwr.written_form_id, iwr.reading))
            .into_grouping_map()
            .collect();
        for iwwf in ignored_word_written_forms {
            if let Some(iw) = word_id_to_ignored_word.get_mut(&iwwf.word_id) {
                let readings = written_form_id_to_readings
                    .remove(&iwwf.written_form_id)
                    .unwrap_or_default();
                iw.written_forms.push(res::IgnoredWordWrittenForm {
                    written_form: iwwf.written_form,
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

pub async fn delete_ignored_word(
    State(state): State<LbrState>,
    Path(word_id): Path<i32>,
    user: Authentication,
) -> LbrResult<()> {
    use crate::schema::ignored_words as iw;
    tracing::info!("Fetching ignored words");

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

query! {
    #[derive(Debug)]
    struct IgnoredWordTranslations {
        word_id: i32 = ignored_words::word_id,
        translations: Vec<Option<String>> = words::translations,
    }
}

query! {
    #[derive(Debug)]
    struct IgnoredWordWrittenForm {
        word_id: i32 = ignored_words::word_id,
        written_form_id: i32 = written_forms::id,
        written_form: String = written_forms::written_form,
    }
}

query! {
    #[derive(Debug)]
    struct IgnoredWordReading {
        word_id: i32 = ignored_words::word_id,
        written_form_id: i32 = written_forms::id,
        reading: String = word_readings::reading,
    }
}
