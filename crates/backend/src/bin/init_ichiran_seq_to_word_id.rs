/// Creates a mapping from ichiran's seqs to LBR word ids.
use diesel::prelude::*;
use eyre::WrapErr;
use lbr_server::{schema::words as w, schema_ichiran as si};
use std::{collections::HashMap, env};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();
    
    let lbr_database_url = env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    let ichiran_database_url =
        env::var("ICHIRAN_DATABASE_URL").wrap_err("Missing ICHIRAN_DATABASE_URL")?;
    let mut lbr_conn = PgConnection::establish(&lbr_database_url)?;
    let mut ichiran_conn = PgConnection::establish(&ichiran_database_url)?;

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
    let contents = bitcode::encode(&ichiran_seq_to_word_id)?;
    std::fs::write("./data/ichiran_seq_to_word_id.bitcode", &contents)?;
    Ok(())
}
