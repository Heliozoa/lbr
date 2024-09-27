//! Creates a mapping from kanji to its potential readings.

use diesel::prelude::*;
use eyre::WrapErr;
use std::env;

pub fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    let mut conn = PgConnection::establish(&database_url)?;

    let kanji_to_readings = lbr_server::domain::japanese::kanji_to_readings(&mut conn)?;
    let contents = bitcode::encode(&kanji_to_readings);
    std::fs::write("./data/kanji_to_readings.bitcode", contents)?;
    Ok(())
}
