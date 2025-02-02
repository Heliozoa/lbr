//! Functions and types related to ichiran.

use diesel::prelude::*;
use std::collections::HashMap;

/// Returns a mapping from ichiran's words to LBR word ids.
// maps (ichiran_seq, word_written_form) => word_id
// e.g. (10159116, 入れる) => 115508
pub fn get_ichiran_word_to_word_id(
    lbr_conn: &mut PgConnection,
    ichiran_conn: &mut PgConnection,
) -> eyre::Result<HashMap<(i32, String, String), i32>> {
    use crate::{
        schema::{word_readings as wr, words as w},
        schema_ichiran as si,
    };

    tracing::info!("Building a mapping from ichiran words to ids");

    let ichiran_seqs = si::entry::table
        .select((si::entry::seq, si::entry::root_p))
        .get_results::<(i32, bool)>(ichiran_conn)?
        .into_iter()
        .collect::<HashMap<_, _>>();
    let conj_seq_via_to_from_vec = si::conjugation::table
        .select((
            si::conjugation::seq,
            si::conjugation::via,
            si::conjugation::from,
        ))
        .get_results::<(i32, Option<i32>, i32)>(ichiran_conn)?;
    let mut conj_seq_via_to_froms = HashMap::<(i32, bool), Vec<i32>>::new();
    for (seq, via, from) in conj_seq_via_to_from_vec {
        let entry = conj_seq_via_to_froms
            .entry((seq, via.is_some()))
            .or_default();
        entry.push(from);
    }
    let jmdict_id_to_words_vec = w::table
        .inner_join(wr::table.on(wr::word_id.eq(w::id)))
        .select((w::jmdict_id, w::id, w::word, wr::reading))
        .get_results::<(i32, i32, String, String)>(lbr_conn)?;
    let mut jmdict_id_to_words = HashMap::<i32, Vec<(i32, String, String)>>::new();
    for (jmdict, id, word, reading) in jmdict_id_to_words_vec {
        let entry = jmdict_id_to_words.entry(jmdict).or_default();
        entry.push((id, word, reading));
    }

    let mut ichiran_word_to_word_id = HashMap::new();
    for ichiran_seq in ichiran_seqs.keys() {
        get_roots(
            &mut ichiran_word_to_word_id,
            *ichiran_seq,
            *ichiran_seq,
            &ichiran_seqs,
            &conj_seq_via_to_froms,
            &jmdict_id_to_words,
        );
    }

    Ok(ichiran_word_to_word_id)
}

fn get_roots(
    ichiran_word_to_word_id: &mut HashMap<(i32, String, String), i32>,
    starting_seq: i32,
    current_seq: i32,
    ichiran_seq_to_root: &HashMap<i32, bool>,
    conj_seq_via_to_froms: &HashMap<(i32, bool), Vec<i32>>,
    jmdict_id_to_words: &HashMap<i32, Vec<(i32, String, String)>>,
) {
    if ichiran_seq_to_root
        .get(&current_seq)
        .copied()
        .unwrap_or_default()
    {
        // is root, add words to map
        if let Some(words) = jmdict_id_to_words.get(&current_seq) {
            for (id, word, reading) in words {
                ichiran_word_to_word_id.insert((starting_seq, word.clone(), reading.clone()), *id);
            }
        }
    }

    // prefer non-via conjugations if any
    if let Some(nexts) = conj_seq_via_to_froms.get(&(current_seq, false)) {
        for next in nexts {
            get_roots(
                ichiran_word_to_word_id,
                starting_seq,
                *next,
                ichiran_seq_to_root,
                conj_seq_via_to_froms,
                jmdict_id_to_words,
            );
        }
    } else if let Some(nexts) = conj_seq_via_to_froms.get(&(current_seq, true)) {
        for next in nexts {
            get_roots(
                ichiran_word_to_word_id,
                starting_seq,
                *next,
                ichiran_seq_to_root,
                conj_seq_via_to_froms,
                jmdict_id_to_words,
            );
        }
    }
}
