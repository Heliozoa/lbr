//! Reusable database query functions.

use diesel::prelude::*;
use std::collections::HashSet;

pub fn ignored_words(conn: &mut PgConnection, user_id: i32) -> eyre::Result<HashSet<i32>> {
    use crate::schema::ignored_words as iw;

    let ignored_words = iw::table
        .select(iw::word_id)
        .filter(iw::user_id.eq(user_id))
        .get_results::<i32>(conn)?
        .into_iter()
        .collect::<HashSet<i32>>();

    Ok(ignored_words)
}
