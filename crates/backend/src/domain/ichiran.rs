//! Functions and types related to ichiran.

use crate::{error::EyreResult, schema::words as w, schema_ichiran as si, LbrPool};
use diesel::prelude::*;
use std::collections::HashMap;

/// Returns a mapping from ichiran's seqs to LBR word ids.
pub async fn get_ichiran_seq_to_word_id(
    lbr_pool: LbrPool,
    ichiran_pool: LbrPool,
) -> eyre::Result<HashMap<i32, i32>> {
    let stw = tokio::task::spawn_blocking(move || {
        let mut lbr_conn = lbr_pool.get()?;
        let mut ichiran_conn = ichiran_pool.get()?;
        let ichiran_seqs = si::entry::table
            .select((si::entry::seq, si::entry::root_p))
            .get_results::<(i32, bool)>(&mut ichiran_conn)?;
        let ichiran_seq_to_conj_from = si::conjugation::table
            .select((si::conjugation::seq, si::conjugation::from))
            .get_results::<(i32, i32)>(&mut ichiran_conn)?
            .into_iter()
            .collect::<HashMap<_, _>>();
        let jmdict_id_to_word_id = w::table
            .select((w::jmdict_id, w::id))
            .get_results::<(i32, i32)>(&mut lbr_conn)?
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
            if let Some(word_id) = jmdict_id_to_word_id.get(&jmdict_seq).copied() {
                ichiran_seq_to_word_id.insert(ichiran_seq, word_id);
            }
        }
        EyreResult::Ok(ichiran_seq_to_word_id)
    })
    .await??;
    Ok(stw)
}
