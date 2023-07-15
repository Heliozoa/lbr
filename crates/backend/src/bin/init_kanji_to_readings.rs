//! Creates a mapping from kanji to its potential readings.

use diesel::prelude::*;
use eyre::WrapErr;
use lbr_server::{
    query,
    schema::{kanji as k, kanji_readings as kr},
};
use std::{collections::HashMap, env};

query! {
    struct KanjiWithReading {
        kanji: String = kanji::chara,
        reading: String = kanji_readings::reading,
    }
}

pub fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    let mut conn = PgConnection::establish(&database_url)?;
    let kanji_with_reading = k::table
        .inner_join(kr::table.on(kr::kanji_id.eq(k::id)))
        .select(KanjiWithReading::as_select())
        .get_results(&mut conn)?;
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for kwr in kanji_with_reading {
        map.entry(kwr.kanji).or_default().push(kwr.reading);
    }
    let contents = bitcode::encode(&map)?;
    std::fs::write("./data/kanji_to_readings.bitcode", &contents)?;
    Ok(())
}
