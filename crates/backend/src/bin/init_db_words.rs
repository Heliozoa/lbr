//! Initialises the words in the database.
//! Clears all previous word-related data.

use diesel::prelude::*;
use eyre::WrapErr;
use jadata::wordfile::{Reading, Wordfile};
use lbr_server::{
    eq,
    schema::{
        kanji as k, word_ichiran as wi, word_kanji as wk, word_readings as wr, words as w,
        written_forms as wf,
    },
    schema_ichiran as is,
    utils::{database::Furigana, diesel::PostgresChunks},
};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
};
use wana_kana::ConvertJapanese;

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let lbr_database_url = std::env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    tracing::info!("This operation will delete old data from {lbr_database_url}, confirm? (y/n)");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    if buf.trim() != "y" {
        tracing::info!("Did not confirm")
    }

    let mut lbr_conn = PgConnection::establish(&lbr_database_url)?;
    let ichiran_database_url =
        std::env::var("ICHIRAN_DATABASE_URL").wrap_err("Missing ICHIRAN_DATABASE_URL")?;
    let mut ichiran_conn = PgConnection::establish(&ichiran_database_url)?;

    tracing::info!("Reading wordfile");
    let wf_path: &str = "./data/jadata/generated/wordfile.json";
    let wf = File::open(wf_path).wrap_err_with(|| format!("Failed to read file at '{wf_path}'"))?;
    tracing::info!("Deserializing wordfile");
    let mut wf: Wordfile = serde_json::from_reader(BufReader::new(wf))?;
    tracing::info!("Processing wordfile");
    process_wordfile(&mut wf);

    tracing::info!("Deleting old data");
    lbr_conn.transaction(|conn| {
        diesel::delete(wi::table).execute(conn)?;
        diesel::delete(wr::table).execute(conn)?;
        diesel::delete(wk::table).execute(conn)?;
        diesel::delete(wf::table).execute(conn)?;
        diesel::delete(w::table).execute(conn)?;
        QueryResult::Ok(())
    })?;

    initialise_ichiran_seq_data(&mut lbr_conn, &mut ichiran_conn)?;

    lbr_conn.transaction(|conn| seed_words(conn, wf))?;
    tracing::info!("finished");

    Ok(())
}

/// A lot of the seqs returned by ichiran are for conjugated versions of words,
/// this function figures out the seq for the root word (which is usually a word in JMdict)
/// for each seq.
fn initialise_ichiran_seq_data(
    lbr_conn: &mut PgConnection,
    ichiran_conn: &mut PgConnection,
) -> eyre::Result<()> {
    tracing::info!("Initialising ichiran seq data");

    let mut ichiran_seq_to_root = is::conjugation::table
        .select((is::conjugation::seq, is::conjugation::from))
        .get_results::<(i32, i32)>(ichiran_conn)?
        .into_iter()
        .collect::<HashMap<_, _>>();
    let mut left_to_process = ichiran_seq_to_root.clone();
    while !left_to_process.is_empty() {
        let mut new = HashMap::new();
        for (&key, &value) in &left_to_process {
            let mut keys = vec![key];
            let mut last = value;
            while let Some(&next_value) = ichiran_seq_to_root.get(&last) {
                keys.push(last);
                last = next_value;
            }
            for key in keys {
                if ichiran_seq_to_root.insert(key, last).is_none() {
                    new.insert(key, last);
                }
            }
        }
        left_to_process = new;
    }
    let wi_values = ichiran_seq_to_root
        .into_iter()
        .map(|(ichiran_seq, root_seq)| eq!(wi, ichiran_seq, root_seq))
        .collect::<Vec<_>>();
    for chunk in wi_values.pg_chunks() {
        diesel::insert_into(wi::table)
            .values(chunk)
            .execute(lbr_conn)?;
    }
    Ok(())
}

// add extra written forms for words that are usually written in kana
fn process_wordfile(wf: &mut Wordfile) {
    for word in &mut wf.words {
        let mut kana_forms = vec![];
        for reading in &mut word.readings {
            if reading.usually_kana {
                kana_forms.push(reading.reading.clone());
            }
        }
        // exclude ones that already have a written form
        let filtered = kana_forms
            .into_iter()
            .filter(|kf| !word.written_forms.iter().any(|wf| wf == kf))
            .collect::<Vec<_>>();
        word.written_forms.extend(filtered);
    }
}

fn seed_words(conn: &mut PgConnection, wf: Wordfile) -> eyre::Result<()> {
    tracing::info!("Seeding words");
    let word_ids = insert_words(conn, &wf)?;
    let wf_ids = insert_written_forms(conn, &wf, &word_ids)?;
    insert_word_kanji(conn, &wf, &wf_ids)?;
    insert_word_readings(conn, &wf, &wf_ids)?;
    Ok(())
}

fn insert_words(conn: &mut PgConnection, wf: &Wordfile) -> eyre::Result<Vec<i32>> {
    tracing::info!("Processing words");
    let w_values = wf
        .words
        .iter()
        .map(|w| {
            (
                w::jmdict_id.eq(w.jmdict_id.map(|id| id as i32)),
                w::translations.eq(&w.meanings),
            )
        })
        .collect::<Vec<_>>();

    tracing::info!("Inserting words");
    let mut w_ids = vec![];
    for chunk in w_values.pg_chunks() {
        w_ids.extend(
            diesel::insert_into(w::table)
                .values(chunk)
                .returning(w::id)
                .get_results::<i32>(conn)?,
        );
    }
    Ok(w_ids)
}

fn insert_written_forms(
    conn: &mut PgConnection,
    wf: &Wordfile,
    word_ids: &[i32],
) -> eyre::Result<Vec<i32>> {
    tracing::info!("Processing written forms");
    let wf_values = word_ids
        .iter()
        .copied()
        .zip(&wf.words)
        .flat_map(|(word_id, word)| word.written_forms.iter().map(move |wf| (word_id, wf)))
        .map(|(word_id, written_form)| eq!(wf, word_id, written_form))
        .collect::<Vec<_>>();

    tracing::info!("Inserting written forms");
    let mut wf_ids = vec![];
    for chunk in wf_values.pg_chunks() {
        wf_ids.extend(
            diesel::insert_into(wf::table)
                .values(chunk)
                .returning(wf::id)
                .get_results::<i32>(conn)?,
        );
    }
    Ok(wf_ids)
}

fn insert_word_kanji(
    conn: &mut PgConnection,
    wf: &Wordfile,
    written_form_ids: &[i32],
) -> eyre::Result<()> {
    tracing::info!("Processing word kanji");
    let kanji_to_id = k::table
        .select((k::chara, k::id))
        .get_results::<(String, i32)>(conn)?
        .into_iter()
        .collect::<HashMap<_, _>>();

    let mut seen = HashSet::new();
    let wk_values = &wf
        .words
        .iter()
        .flat_map(|w| w.written_forms.iter())
        .zip(written_form_ids)
        .flat_map(|(wf, wf_id)| lbr::kanji_from_word(wf).map(move |kanji| (wf_id, kanji)))
        .filter(|tuple| seen.insert(*tuple))
        .map(|(wf_id, kanji)| (wf_id, kanji_to_id.get(kanji).unwrap()))
        .map(|(written_form_id, kanji_id)| eq!(wk, written_form_id, kanji_id))
        .collect::<Vec<_>>();

    tracing::info!("Inserting word kanji");
    for chunk in wk_values.pg_chunks() {
        diesel::insert_into(wk::table).values(chunk).execute(conn)?;
    }
    Ok(())
}

fn insert_word_readings(
    conn: &mut PgConnection,
    wf: &Wordfile,
    written_form_ids: &[i32],
) -> eyre::Result<()> {
    tracing::info!("Processing word readings");
    let wr_values = written_form_ids
        .iter()
        .copied()
        .zip(wf.words.iter().flat_map(|w| &w.readings))
        .map(
            |(
                written_form_id,
                Reading {
                    reading,
                    furigana,
                    usually_kana,
                },
            )| {
                let reading_katakana = reading.to_katakana();
                let mut word_idx = 0;
                let mut reading_idx = 0;
                let furigana = furigana
                    .iter()
                    .map(|f| {
                        if word_idx < f.start_idx {
                            reading_idx += f.start_idx - word_idx;
                        }
                        word_idx = f.end_idx;
                        let reading_start_idx = reading_idx;
                        let reading_end_idx = reading_start_idx + f.furigana.len();
                        reading_idx = reading_end_idx;
                        Furigana {
                            reading_start_idx: reading_start_idx.try_into().expect("invalid idx"),
                            reading_end_idx: reading_end_idx.try_into().expect("invalid idx"),
                            word_start_idx: f.start_idx.try_into().expect("invalid idx"),
                            word_end_idx: f.end_idx.try_into().expect("invalid idx"),
                        }
                    })
                    .collect::<Vec<_>>();
                eq!(
                    wr::written_form_id,
                    wr::reading,
                    wr::reading_katakana,
                    wr::furigana,
                    wr::usually_kana
                )
            },
        )
        .collect::<Vec<_>>();

    tracing::info!("Inserting word readings");
    for chunk in wr_values.pg_chunks() {
        diesel::insert_into(wr::table).values(chunk).execute(conn)?;
    }
    Ok(())
}
