use diesel::prelude::*;
use eyre::Context;
use jadata::{
    jmdict::JMdict,
    jmdict_furigana::JmdictFurigana,
    kanji_extra::{ExtraKanji, KanjiExtra},
    kanji_names::KanjiNames,
    kanji_similar::KanjiSimilar,
    kanjidic::Kanjidic2,
    kradfile::Kradfile,
};
use lbr_server::utils::database::Furigana;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
};

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").wrap_err("Missing DATABASE_URL")?;
    let mut conn = PgConnection::establish(&database_url)?;

    let args = std::env::args().collect::<Vec<_>>();

    let kd2 = &args[1];
    tracing::info!("Opening {kd2}");
    let kd2 = File::open(kd2).context("Failed to open file")?;
    tracing::info!("Deserializing");
    let kd2: Kanjidic2 =
        serde_xml_rs::from_reader(BufReader::new(kd2)).context("Failed to deserialize data")?;

    let kf = &args[2];
    tracing::info!("Opening {kf}");
    let kf = File::open(kf).context("Failed to open file")?;
    tracing::info!("Deserializing");
    let kf: Kradfile = Kradfile::from(kf).context("Failed to deserialize data")?;

    let kn = &args[3];
    tracing::info!("Opening {kn}");
    let kn = File::open(kn).context("Failed to open file")?;
    tracing::info!("Deserializing");
    let kn: KanjiNames = serde_json::from_reader(kn).context("Failed to deserialize data")?;

    let ks = &args[4];
    tracing::info!("Opening {ks}");
    let ks = File::open(ks).context("Failed to open file")?;
    tracing::info!("Deserializing");
    let ks: KanjiSimilar = serde_json::from_reader(ks).context("Failed to deserialize data")?;

    let ke_path = &args[5];
    tracing::info!("Opening {ke_path}");
    let ke = File::open(ke_path).context("Failed to open file")?;
    tracing::info!("Deserializing");
    let mut ke: KanjiExtra = serde_json::from_reader(ke).context("Failed to deserialize data")?;

    let jmdict = &args[6];
    tracing::info!("Opening {jmdict}");
    let jmdict = File::open(jmdict).context("Failed to open file")?;
    tracing::info!("Deserializing");
    let jmdict: JMdict =
        serde_xml_rs::from_reader(BufReader::new(jmdict)).context("Failed to deserialize data")?;

    let jmdict_furigana = &args[7];
    tracing::info!("Opening {jmdict_furigana}");
    let jmdict_furigana = File::open(jmdict_furigana).context("Failed to open file")?;
    tracing::info!("Deserializing");
    let jmdict_furigana: JmdictFurigana = serde_json::from_reader(BufReader::new(jmdict_furigana))
        .context("Failed to deserialize data")?;

    tracing::info!("Updating extra kanji");
    update_kanji_extra(&kd2, &jmdict, &mut ke, ke_path)?;

    conn.transaction(|conn| {
        tracing::info!("Starting transaction");
        update_kanji(conn, &kd2, &ke, &kf, &kn, &ks).context("Failed to update kanji")?;
        update_words(conn, &jmdict, &jmdict_furigana).context("Failed to update words")?;
        eyre::Ok(())
    })?;
    tracing::info!("Finished transaction");

    Ok(())
}

fn update_kanji_extra(
    kd2: &Kanjidic2,
    jmdict: &JMdict,
    ke: &mut KanjiExtra,
    ke_path: &str,
) -> eyre::Result<()> {
    let kanjidic_kanji = kd2.character.iter().map(|c| c.literal.as_str());
    let extra_kanji = ke.kanji_extra.iter().map(|ke| ke.chara.as_str());
    let included_kanji = kanjidic_kanji.chain(extra_kanji).collect::<HashSet<_>>();

    let jmdict_kanji = jmdict
        .entry
        .iter()
        .flat_map(|e| &e.k_ele)
        .map(|kele| &kele.keb)
        .flat_map(|keb| lbr::kanji_from_word(keb))
        .collect::<HashSet<_>>();

    let missing_kanji = jmdict_kanji
        .difference(&included_kanji)
        .map(|missing_kanji| ExtraKanji {
            chara: missing_kanji.to_string(),
            meanings: Vec::new(),
            components: Vec::new(),
            readings: Vec::new(),
        })
        .collect::<Vec<_>>();
    if !missing_kanji.is_empty() {
        ke.kanji_extra.extend(missing_kanji);
        ke.kanji_extra.sort_by(|a, b| a.chara.cmp(&b.chara));
        let mut ke_file = File::create(ke_path).context("Failed to create file")?;
        serde_json::to_writer_pretty(&mut ke_file, &ke).context("Failed to serialize")?;
    }

    Ok(())
}

fn update_kanji(
    conn: &mut PgConnection,
    kd2: &Kanjidic2,
    ke: &KanjiExtra,
    kf: &Kradfile,
    kn: &KanjiNames,
    ks: &KanjiSimilar,
) -> eyre::Result<()> {
    use lbr_server::schema::{kanji as k, kanji_readings as kr, kanji_similar as ks};

    tracing::info!("Updating kanji");
    let existing_kanji = k::table
        .select((k::chara, k::id))
        .get_results::<(String, i32)>(conn)?
        .into_iter()
        .collect::<HashMap<String, i32>>();

    // the kanji readings are created fresh each time so we delete them first
    diesel::delete(kr::table)
        .execute(conn)
        .context("Failed to delete kanji readings")?;

    for kanji in &kd2.character {
        tracing::info!("Processing kanji {}", kanji.literal);

        let existing_kanji = existing_kanji.get(&kanji.literal).copied();
        let name = kn.kanji_names.get(&kanji.literal);
        let meanings = kanji
            .reading_meaning
            .as_ref()
            .map(|rm| rm.rmgroup.as_slice())
            .unwrap_or_default()
            .iter()
            .flat_map(|rm| &rm.meaning)
            .filter(|m| m.m_lang.is_none())
            .map(|m| &m.value)
            .collect::<Vec<_>>();
        let components = kf
            .kanji_to_components
            .get(&kanji.literal)
            .map(|v| v.as_slice())
            .unwrap_or_default();
        let kanji_id = if let Some(existing_kanji) = existing_kanji {
            // update
            diesel::update(k::table.filter(k::id.eq(existing_kanji)))
                .set((
                    k::name.eq(name),
                    k::meanings.eq(meanings),
                    k::components.eq(components),
                ))
                .execute(conn)
                .context("Failed to update kanji")?;
            existing_kanji
        } else {
            // create new
            diesel::insert_into(k::table)
                .values((
                    k::chara.eq(&kanji.literal),
                    k::name.eq(name),
                    k::meanings.eq(meanings),
                    k::components.eq(components),
                ))
                .returning(k::id)
                .get_result(conn)
                .context("Failed to create new kanji")?
        };

        let mut new_kanji_readings = Vec::new();
        for reading in kanji
            .reading_meaning
            .as_ref()
            .map(|rm| rm.rmgroup.as_slice())
            .unwrap_or_default()
            .iter()
            .flat_map(|rmg| &rmg.reading)
        {
            let (reading, okurigana) = reading
                .value
                .split_once(".")
                .map(|(r, o)| (r, Some(o)))
                .unwrap_or((&reading.value, None));
            new_kanji_readings.push((
                kr::kanji_id.eq(kanji_id),
                kr::reading.eq(reading),
                kr::okurigana.eq(okurigana),
            ));
        }
        for chunk in new_kanji_readings.chunks(255) {
            diesel::insert_into(kr::table)
                .values(chunk)
                .execute(conn)
                .context("Failed to create new kanji readings")?;
        }
    }

    tracing::info!("Processing similar kanji");
    diesel::delete(ks::table)
        .execute(conn)
        .context("Failed to delete similar kanji")?;
    let kanji_to_id = k::table
        .select((k::chara, k::id))
        .get_results(conn)?
        .into_iter()
        .collect::<HashMap<String, i32>>();
    let mut new_similar_kanji = Vec::new();
    for (kanji, similar) in &ks.kanji_similar {
        for similar in similar {
            let kanji_id = kanji_to_id.get(kanji).copied().unwrap();
            let similar_id = kanji_to_id.get(similar).copied().unwrap();
            let lower_kanji_id = kanji_id.min(similar_id);
            let higher_kanji_id = kanji_id.max(similar_id);
            new_similar_kanji.push((
                ks::lower_kanji_id.eq(lower_kanji_id),
                ks::higher_kanji_id.eq(higher_kanji_id),
            ));
        }
    }
    for chunk in new_similar_kanji.chunks(255) {
        diesel::insert_into(ks::table).values(chunk).execute(conn)?;
    }

    tracing::info!("Processing extra kanji");
    let mut new_extra_kanji_readings = Vec::new();
    for ke in &ke.kanji_extra {
        let name = kn.kanji_names.get(&ke.chara);
        let meanings = &ke.meanings;
        let components = &ke.components;

        let kanji_id = if let Some(existing_kanji) = kanji_to_id.get(&ke.chara).copied() {
            // update
            diesel::update(k::table.filter(k::id.eq(existing_kanji)))
                .set((
                    k::name.eq(name),
                    k::meanings.eq(meanings),
                    k::components.eq(components),
                ))
                .execute(conn)
                .context("Failed to update kanji")?;
            existing_kanji
        } else {
            // create new
            diesel::insert_into(k::table)
                .values((
                    k::chara.eq(&ke.chara),
                    k::name.eq(name),
                    k::meanings.eq(meanings),
                    k::components.eq(components),
                ))
                .returning(k::id)
                .get_result(conn)
                .context("Failed to create new kanji")?
        };

        for reading in &ke.readings {
            let (reading, okurigana) = reading
                .split_once(".")
                .map(|(r, o)| (r, Some(o)))
                .unwrap_or((reading, None));
            new_extra_kanji_readings.push((
                kr::kanji_id.eq(kanji_id),
                kr::reading.eq(reading),
                kr::okurigana.eq(okurigana),
            ));
        }
    }
    for chunk in new_extra_kanji_readings.chunks(255) {
        diesel::insert_into(kr::table)
            .values(chunk)
            .execute(conn)
            .context("Failed to create new kanji readings")?;
    }

    Ok(())
}

fn update_words(
    conn: &mut PgConnection,
    jmdict: &JMdict,
    jmdict_furigana: &JmdictFurigana,
) -> eyre::Result<()> {
    use lbr_server::schema::{kanji as k, word_kanji as wk, word_readings as wr, words as w};

    tracing::info!("Updating words");
    let furigana = process_furigana(jmdict_furigana);
    let existing_words = w::table
        .select((w::jmdict_id, w::word, w::id))
        .get_results::<(i32, String, i32)>(conn)?;
    let existing_words = existing_words
        .iter()
        .map(|(jmdict_id, word, id)| ((*jmdict_id, word.as_str()), *id))
        .collect::<HashMap<(i32, &str), i32>>();
    for entry in &jmdict.entry {
        let jmdict_id = entry
            .ent_seq
            .parse::<i32>()
            .context("Failed to parse jmdict seq")?;
        tracing::info!("Processing entry {}", jmdict_id);

        // few things to consider here:
        // a word may not have any kanji entries, in which case the readings are the written forms
        // a kanji entry may be marked as rare, if all kanji entries are rare we should again treat the readings as the written forms
        // a meaning of a word can be marked as "usually kana", these should also get their own entries with the reading as the written form
        let mut new_words = HashMap::<&str, Vec<NewWord>>::new();
        struct NewWord<'a> {
            word: &'a str,
            reading: &'a str,
            translations: Vec<&'a str>,
        }
        // skip search only forms
        for k_ele in entry
            .k_ele
            .iter()
            .filter(|k_ele| k_ele.ke_inf.iter().all(|ke_inf| ke_inf != "sK"))
        {
            // only include reading elements that do not exclude the kanji element
            for r_ele in entry
                .r_ele
                .iter()
                .filter(|r_ele| r_ele.re_restr.iter().all(|restr| restr != &k_ele.keb))
            {
                let word = &k_ele.keb;
                let reading = &r_ele.reb;
                let translations = entry
                    .sense
                    .iter()
                    .filter(|s| s.stagk.is_empty() || s.stagk.contains(word))
                    .filter(|s| s.stagr.is_empty() || s.stagr.contains(reading))
                    .flat_map(|s| s.gloss.iter())
                    .filter(|g| g.lang.is_none())
                    .map(|g| g.value.as_str())
                    .collect::<Vec<_>>();
                let entry = new_words.entry(&k_ele.keb).or_default();
                entry.push(NewWord {
                    word,
                    reading,
                    translations,
                });
            }
        }
        // check if the entry has no kanji elements
        if entry.k_ele.is_empty() {
            // add all reading elements as their own words as well
            for r_ele in &entry.r_ele {
                let word = &r_ele.reb;
                let reading = &r_ele.reb;
                let translations = entry
                    .sense
                    .iter()
                    .filter(|s| s.stagk.is_empty())
                    .filter(|s| s.stagr.is_empty() || s.stagr.contains(reading))
                    .flat_map(|s| s.gloss.iter())
                    .filter(|g| g.lang.is_none())
                    .map(|g| g.value.as_str())
                    .collect::<Vec<_>>();
                let entry = new_words.entry(&r_ele.reb).or_default();
                entry.push(NewWord {
                    word,
                    reading,
                    translations,
                });
            }
        }
        // check for rare kanji elements
        for k_ele in entry
            .k_ele
            .iter()
            .filter(|k_ele| k_ele.ke_inf.iter().any(|ke_inf| ke_inf == "rK"))
        {
            // add all reading elements that apply to this kanji element as their own words as well
            for r_ele in &entry.r_ele {
                let word = &k_ele.keb;
                let reading = &r_ele.reb;
                let translations = entry
                    .sense
                    .iter()
                    .filter(|s| s.stagk.is_empty() || s.stagk.contains(word))
                    .filter(|s| s.stagr.is_empty() || s.stagr.contains(reading))
                    .flat_map(|s| s.gloss.iter())
                    .filter(|g| g.lang.is_none())
                    .map(|g| g.value.as_str())
                    .collect::<Vec<_>>();
                let entry = new_words.entry(&r_ele.reb).or_default();
                entry.push(NewWord {
                    word,
                    reading,
                    translations,
                });
            }
        }
        // check for translations usually in kana
        for sense in &entry.sense {
            if sense.s_inf.iter().any(|s_inf| s_inf == "uk") {
                for r_ele in &entry.r_ele {
                    let word = &r_ele.reb;
                    let reading = &r_ele.reb;
                    let translations = entry
                        .sense
                        .iter()
                        .filter(|s| s.stagk.is_empty() || s.stagk.contains(word))
                        .filter(|s| s.stagr.is_empty() || s.stagr.contains(reading))
                        .flat_map(|s| s.gloss.iter())
                        .filter(|g| g.lang.is_none())
                        .map(|g| g.value.as_str())
                        .collect::<Vec<_>>();
                    let entry = new_words.entry(&r_ele.reb).or_default();
                    entry.push(NewWord {
                        word,
                        reading,
                        translations,
                    });
                }
            }
        }

        for new_word in new_words.values().flatten() {
            let word = new_word.word;
            let reading = new_word.reading;
            let translations = &new_word.translations;
            tracing::info!("Processing word {} ({})", word, reading);

            let furigana = furigana.get(&(word, reading)).cloned().unwrap_or_default();

            let existing_word = existing_words.get(&(jmdict_id, word)).copied();
            if existing_word.is_some() {
                // nothing to do here
            } else {
                // create new
                let word_id = diesel::insert_into(w::table)
                    .values((w::jmdict_id.eq(jmdict_id), w::word.eq(word)))
                    .returning(w::id)
                    .get_result::<i32>(conn)
                    .context("Failed to create new word")?;
                // for new words, we also insert the word kanji and readings
                diesel::insert_into(wr::table)
                    .values((
                        wr::word_id.eq(word_id),
                        wr::reading.eq(reading),
                        wr::furigana.eq(furigana),
                        wr::translations.eq(translations),
                    ))
                    .execute(conn)?;
                let mut word_kanji = Vec::new();
                for kanji in lbr::kanji_from_word(word) {
                    let kanji_id = k::table
                        .filter(k::chara.eq(kanji))
                        .select(k::id)
                        .get_result::<i32>(conn)
                        .with_context(|| format!("Failed to fetch kanji {kanji}"))?;
                    word_kanji.push((wk::word_id.eq(word_id), wk::kanji_id.eq(kanji_id)));
                }
                diesel::insert_into(wk::table)
                    .values(word_kanji)
                    .on_conflict_do_nothing()
                    .execute(conn)
                    .context("Failed to create new word kanji")?;
            }
        }
    }

    Ok(())
}

// (word, reading) -> furigana
fn process_furigana(furigana: &JmdictFurigana) -> HashMap<(&str, &str), Vec<Furigana>> {
    furigana
        .iter()
        .map(|f| {
            let key = (f.text.as_str(), f.reading.as_str());
            let mut word_start_idx = 0;
            let mut reading_start_idx = 0;
            let mut furigana = vec![];
            for ruby in &f.furigana {
                let word_end_idx = word_start_idx + ruby.ruby.len() as i32;
                if let Some(rt) = &ruby.rt {
                    let reading_end_idx = reading_start_idx + rt.len() as i32;
                    furigana.push(Furigana {
                        word_start_idx,
                        word_end_idx,
                        reading_start_idx,
                        reading_end_idx,
                    });
                    reading_start_idx = reading_end_idx;
                } else {
                    // no rt means the section of the word and the reading are the same,
                    reading_start_idx += ruby.ruby.len() as i32;
                }
                word_start_idx = word_end_idx;
            }
            (key, furigana)
        })
        .collect()
}
