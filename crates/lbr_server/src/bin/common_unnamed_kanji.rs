//! Creates a mapping from ichiran's seqs to LBR word ids.

use diesel::prelude::*;
use eyre::WrapErr;
use jadata::kanji_names::KanjiNames;
use std::{collections::HashMap, env, io::stdin};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let lbr_database_url = env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    let mut lbr_conn = PgConnection::establish(&lbr_database_url)?;
    print_common_unnamed_kanji(&mut lbr_conn)?;
    Ok(())
}

fn print_common_unnamed_kanji(lbr_conn: &mut PgConnection) -> eyre::Result<()> {
    use lbr_server::schema::{kanji as k, sentence_words as sw, word_kanji as wk, words as w};

    let mut kanji_words: HashMap<String, Vec<(String, String, String)>> = HashMap::new();
    k::table
        .inner_join(wk::table.on(wk::kanji_id.eq(k::id)))
        .inner_join(w::table.on(w::id.eq(wk::word_id)))
        .select((k::chara, k::meanings, w::word, w::translations))
        .get_results::<(String, Vec<Option<String>>, String, Vec<Option<String>>)>(lbr_conn)?
        .into_iter()
        .for_each(|(k, km, w, t)| {
            let entry = kanji_words.entry(k).or_default();
            let t = t.into_iter().flatten().collect::<Vec<_>>().join(", ");
            let m = km.into_iter().flatten().collect::<Vec<_>>().join(", ");
            entry.push((m, w, t));
        });

    let common_unnamed_kanji: Vec<(String, i64)> = k::table
        .inner_join(wk::table.on(wk::kanji_id.eq(k::id)))
        .inner_join(w::table.on(w::id.eq(wk::word_id)))
        .inner_join(sw::table.on(sw::word_id.eq(w::id.nullable())))
        .filter(k::name.is_null())
        .group_by(k::chara)
        .order(diesel::dsl::count_distinct(sw::sentence_id).desc())
        .select((k::chara, diesel::dsl::count_distinct(sw::sentence_id)))
        .get_results::<(String, i64)>(lbr_conn)?;

    let file = std::fs::File::open("/home/sasami-san/Dev/lbr/crates/jadata/data/kanji_names.json")
        .unwrap();
    let s: KanjiNames = serde_json::from_reader(file).unwrap();

    let mut names: HashMap<String, String> = HashMap::new();
    for (chara, sentences) in common_unnamed_kanji
        .into_iter()
        .filter(|c| !s.kanji_names.contains_key(&c.0))
    {
        let kw = kanji_words
            .get(&chara)
            .map(Vec::as_slice)
            .unwrap_or_default();

        let meanings = kw.first().map(|t| t.0.as_str()).unwrap_or_default();
        println!("{chara} {sentences} {meanings}");
        for (_meanings, word, translations) in kw.iter().take(16) {
            println!("    {word}: {translations:?}");
        }
        print!("input: ");
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        if line.trim().is_empty() {
            break;
        }
        names.insert(chara, line.trim().to_string());
    }
    println!("{names:#?}");
    Ok(())
}
