//! Creates a mapping from ichiran's seqs to LBR word ids.

use diesel::prelude::*;
use eyre::WrapErr;
use std::env;

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let lbr_database_url = env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    let ichiran_database_url =
        env::var("ICHIRAN_DATABASE_URL").wrap_err("Missing ICHIRAN_DATABASE_URL")?;
    let mut lbr_conn = PgConnection::establish(&lbr_database_url)?;
    let mut ichiran_conn = PgConnection::establish(&ichiran_database_url)?;

    let ichiran_seq_to_word_id =
        lbr_server::domain::ichiran::get_ichiran_seq_to_word_id(&mut lbr_conn, &mut ichiran_conn)?;
    let contents = bitcode::encode(&ichiran_seq_to_word_id)?;
    std::fs::write("./data/ichiran_seq_to_word_id.bitcode", contents)?;
    Ok(())
}
