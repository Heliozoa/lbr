//! Creates a mapping from ichiran's seqs to LBR word ids.

use diesel::prelude::*;
use eyre::WrapErr;
use jadata::kanji_names::KanjiNames;
use std::{
    collections::{HashMap, HashSet},
    env,
    io::{stdin, stdout, Write},
};

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

    let common_unnamed_kanji_db: Vec<(String, i64)> = k::table
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
    let kanji_names_json: KanjiNames = serde_json::from_reader(file).unwrap();

    let mut dupe_check = HashSet::new();
    for n in kanji_names_json.kanji_names.values() {
        if dupe_check.contains(n) {
            println!("dupe {n}");
        }
        dupe_check.insert(n);
    }

    let kanji_names_db = k::table
        .select((k::chara, k::name))
        .get_results::<(String, Option<String>)>(lbr_conn)?;

    for (kanji, name) in &kanji_names_db {
        if let Some(name) = name.as_ref() {
            if !kanji_names_json.kanji_names.contains_key(kanji) {
                println!("missing \"{}\": \"{}\"", kanji, name)
            }
        }
    }

    let existing_names = kanji_names_db
        .into_iter()
        .map(|knd| knd.1)
        .flatten()
        .chain(kanji_names_json.kanji_names.values().map(String::from))
        .collect::<HashSet<_>>();

    let common_unnamed_kanji = common_unnamed_kanji_db
        .into_iter()
        .filter(|c| !kanji_names_json.kanji_names.contains_key(&c.0))
        .collect::<Vec<_>>();

    let mut names: HashMap<String, String> = HashMap::new();
    println!("unnamed kanji: {}", common_unnamed_kanji.len());
    'outer: for (chara, sentences) in common_unnamed_kanji {
        let mut kw = kanji_words
            .get(&chara)
            .map(Vec::as_slice)
            .unwrap_or_default()
            .to_vec();

        kw.sort_unstable_by_key(|w| {
            w.1.chars()
                .filter(|c| wana_kana::utils::is_char_kanji(*c))
                .count()
                * 10_000
                + w.1.len()
        });

        let meanings = kw.first().map(|t| t.0.as_str()).unwrap_or_default();
        println!("{chara} {sentences} {meanings}");
        for (_meanings, word, translations) in kw.iter().take(16) {
            println!("    {word}: {translations:?}");
        }
        loop {
            print!("input: ");
            stdout().flush().unwrap();
            let mut line = String::new();
            stdin().read_line(&mut line).unwrap();
            if line.trim().is_empty() {
                break 'outer;
            }
            let name = line.trim().to_string();
            if existing_names.contains(&name) {
                println!("\nalready exists");
                continue;
            } else {
                names.insert(chara, name);
                break;
            }
        }
    }
    println!("{names:#?}");
    for (kanji, name) in names {
        diesel::update(k::table)
            .set(k::name.eq(name))
            .filter(k::chara.eq(kanji))
            .execute(lbr_conn)?;
    }
    Ok(())
}
