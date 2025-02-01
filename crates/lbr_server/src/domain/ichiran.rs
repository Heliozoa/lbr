//! Functions and types related to ichiran.

use crate::{schema::words as w, schema_ichiran as si};
use diesel::prelude::*;
use std::collections::HashMap;

/// Returns a mapping from ichiran's words to LBR word ids.
// maps (ichiran_seq, word_written_form) => word_id
// e.g. (10159116, 入れる) => 115508
pub fn get_ichiran_word_to_word_id(
    lbr_conn: &mut PgConnection,
    ichiran_conn: &mut PgConnection,
) -> eyre::Result<HashMap<(i32, String), i32>> {
    tracing::info!("Building a mapping from ichiran words to ids");

    let ichiran_seqs = si::entry::table
        .select((si::entry::seq, si::entry::root_p))
        .get_results::<(i32, bool)>(ichiran_conn)?
        .into_iter()
        .collect::<HashMap<_, _>>();
    let conj_seq_via_to_from = si::conjugation::table
        .filter(si::conjugation::via.is_null())
        .select((
            si::conjugation::seq,
            si::conjugation::via,
            si::conjugation::from,
        ))
        .get_results::<(i32, Option<i32>, i32)>(ichiran_conn)?
        .into_iter()
        .map(|(s, v, f)| ((s, v.is_some()), f))
        .collect::<HashMap<_, _>>();
    let jmdict_id_to_words_vec = w::table
        .select((w::jmdict_id, w::word, w::id))
        .get_results::<(i32, String, i32)>(lbr_conn)?;
    let mut jmdict_id_to_words = HashMap::<i32, Vec<(i32, String)>>::new();
    for (jmdict, word, id) in jmdict_id_to_words_vec {
        let entry = jmdict_id_to_words.entry(jmdict).or_default();
        entry.push((id, word));
    }

    let mut ichiran_word_to_word_id = HashMap::new();
    for (ichiran_seq, root_p) in ichiran_seqs.iter().map(|(s, r)| (*s, *r)) {
        let mut current_seq = ichiran_seq;
        loop {
            // check if current seq is root
            if ichiran_seqs.get(&current_seq).copied().unwrap_or_default() {
                // if so, add it to the map
                if let Some(words) = jmdict_id_to_words.get(&current_seq) {
                    for (id, word) in words {
                        ichiran_word_to_word_id.insert((ichiran_seq, word.clone()), *id);
                    }
                }
            }

            // check conjugations
            // non-via conjugations are used first
            // todo: recurse here and use both?
            if let Some(root) = conj_seq_via_to_from.get(&(current_seq, false)).copied() {
                current_seq = root;
            } else if let Some(root) = conj_seq_via_to_from.get(&(current_seq, true)).copied() {
                current_seq = root
            } else {
                // no more conjugations
                break;
            }
        }
    }

    Ok(ichiran_word_to_word_id)
}
