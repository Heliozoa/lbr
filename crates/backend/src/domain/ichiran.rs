//! Functions and types related to ichiran.

use crate::{schema::words as w, schema_ichiran as si};
use diesel::prelude::*;
use std::collections::HashMap;

/// Returns a mapping from ichiran's seqs to LBR word ids.
pub fn get_ichiran_seq_to_word_id(
    lbr_conn: &mut PgConnection,
    ichiran_conn: &mut PgConnection,
) -> eyre::Result<HashMap<i32, i32>> {
    let ichiran_seqs = si::entry::table
        .select((si::entry::seq, si::entry::root_p))
        .get_results::<(i32, bool)>(ichiran_conn)?;
    let ichiran_seq_to_conj_from = si::conjugation::table
        .select((si::conjugation::seq, si::conjugation::from))
        .get_results::<(i32, i32)>(ichiran_conn)?
        .into_iter()
        .collect::<HashMap<_, _>>();
    let jmdict_id_to_word_id = w::table
        .select((w::jmdict_id, w::id))
        .get_results::<(Option<i32>, i32)>(lbr_conn)?
        .into_iter()
        .collect::<HashMap<_, _>>();
    let mut ichiran_seq_to_word_id = HashMap::new();
    for (ichiran_seq, root_p) in ichiran_seqs {
        let mut root_seq = ichiran_seq;
        if root_p {
            root_seq = ichiran_seq;
        } else {
            while let Some(root) = ichiran_seq_to_conj_from.get(&root_seq).copied() {
                root_seq = root;
            }
        }
        let jmdict_seq = root_seq;
        if let Some(word_id) = jmdict_id_to_word_id.get(&Some(jmdict_seq)).copied() {
            ichiran_seq_to_word_id.insert(ichiran_seq, word_id);
        }
    }
    Ok(ichiran_seq_to_word_id)
}
