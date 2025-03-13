//! Functions and types related to ichiran.

use diesel::prelude::*;
use std::collections::HashMap;

/// Returns a mapping from word ids to their meanings.
pub fn get_word_to_meanings(conn: &mut PgConnection) -> eyre::Result<HashMap<i32, Vec<String>>> {
    use crate::schema::words as w;

    tracing::info!("Building a mapping from word ids to meanings");

    let mut word_to_meanings = HashMap::<i32, Vec<String>>::new();
    w::table
        .select((w::id, w::translations))
        .get_results::<(i32, Vec<Option<String>>)>(conn)?
        .into_iter()
        .for_each(|(id, tr)| {
            let entry = word_to_meanings.entry(id).or_default();
            for tr in tr.into_iter().flatten() {
                entry.push(tr);
            }
        });
    Ok(word_to_meanings)
}
