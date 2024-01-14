//! Initialises the kanji in the database from the kanjifile.
//! Clears all previous kanji-related data.

use diesel::prelude::*;
use eyre::{Context, ContextCompat};
use jadata::kanjifile::{Kanji, Kanjifile};
use lbr_server::{
    eq,
    schema::{kanji as k, kanji_readings as kr, kanji_similar as ks, word_kanji as wk},
    utils::diesel::PostgresChunks,
};
use std::{collections::HashMap, fs::File, io::BufReader};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    tracing::info!("This operation will delete old data from {database_url}, confirm? (y/n)");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    if buf.trim() != "y" {
        tracing::info!("Did not confirm")
    }

    let mut conn = PgConnection::establish(&database_url)?;

    tracing::info!("Reading kanjifile");
    let kf_path = "./data/jadata/generated/kanjifile.json";
    let kf = File::open(kf_path).wrap_err_with(|| format!("Failed to read file at '{kf_path}'"))?;
    tracing::info!("Deserializing kanjifile");
    let kf: Kanjifile = serde_json::from_reader(BufReader::new(kf))?;

    tracing::info!("Deleting old data");
    conn.transaction(|conn| {
        diesel::delete(wk::table).execute(conn)?;
        diesel::delete(ks::table).execute(conn)?;
        diesel::delete(kr::table).execute(conn)?;
        diesel::delete(k::table).execute(conn)?;
        QueryResult::Ok(())
    })?;

    conn.transaction(|conn| seed_kanji(conn, kf))?;
    tracing::info!("Finished");

    Ok(())
}

fn seed_kanji(conn: &mut PgConnection, kf: Kanjifile) -> eyre::Result<()> {
    tracing::info!("Seeding kanji");
    let kanji_ids = insert_kanji(conn, &kf)?;
    insert_kanji_similar(conn, &kf.kanji, &kanji_ids)?;
    Ok(())
}

fn insert_kanji(conn: &mut PgConnection, kf: &Kanjifile) -> eyre::Result<Vec<i32>> {
    tracing::info!("Processing kanji");
    let mut k_values = vec![];
    for k in kf.kanji.iter() {
        let chara = &k.kanji;
        let name = &k.name;
        let meanings = &k.meanings;
        let components = &k.components;
        k_values.push(eq!(k, chara, name, meanings, components));
    }
    tracing::info!("Inserting kanji");
    let k_ids = diesel::insert_into(k::table)
        .values(k_values)
        .returning(k::id)
        .get_results::<i32>(conn)?;
    Ok(k_ids)
}

fn insert_kanji_similar(
    conn: &mut PgConnection,
    kanji_list: &[Kanji],
    kanji_ids: &[i32],
) -> eyre::Result<()> {
    tracing::info!("Processing similar kanji");
    let mut ks_values = vec![];
    let kanji_to_id = kanji_list
        .iter()
        .zip(kanji_ids)
        .map(|(k, id)| (&k.kanji, id))
        .collect::<HashMap<_, _>>();
    for (kanji, kanji_id) in kanji_list.iter().zip(kanji_ids) {
        for similar in &kanji.similar {
            let similar_id = kanji_to_id
                .get(similar)
                .wrap_err_with(|| format!("Invalid similar kanji {similar}"))?;
            let lower_kanji_id = kanji_id.min(similar_id);
            let higher_kanji_id = kanji_id.max(similar_id);
            ks_values.push(eq!(ks, lower_kanji_id, higher_kanji_id));
        }
    }
    tracing::info!("Inserting similar kanji");
    for chunk in ks_values.pg_chunks() {
        diesel::insert_into(ks::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(conn)?;
    }
    Ok(())
}
