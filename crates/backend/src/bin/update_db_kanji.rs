//! Updates the database kanji with the supplemental kanjifiles.

use diesel::prelude::*;
use eyre::Context;
use jadata::kanjifile::Kanjifile;
use std::{fs::File, io::BufReader};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    let mut conn = PgConnection::establish(&database_url)?;

    tracing::info!("Reading kanjifile");
    let kf_path = "./crates/jadata/generated/kanjifile.json";
    let kf = File::open(kf_path).wrap_err_with(|| format!("Failed to read file at '{kf_path}'"))?;
    tracing::info!("Deserializing kanjifile");
    let kf: Kanjifile = serde_json::from_reader(BufReader::new(kf))?;

    conn.transaction(|conn| update_kanji(conn, kf))?;
    tracing::info!("Finished");

    Ok(())
}

fn update_kanji(_conn: &mut PgConnection, _kf: Kanjifile) -> eyre::Result<()> {
    todo!()
}
