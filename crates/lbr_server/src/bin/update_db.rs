use diesel::prelude::*;
use eyre::{Context, ContextCompat};
use jadata::{
    jmdict::{Entry, JMdict},
    kanji_extra::{ExtraKanji, KanjiExtra},
    kanji_names::KanjiNames,
    kanji_similar::KanjiSimilar,
    kanjidic::Kanjidic2,
    kradfile::Kradfile,
    similar_kanji,
};
use lbr_server::domain;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    path::Path,
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

    let sk = &args[7];
    tracing::info!("Opening {sk}");
    let sk = similar_kanji::parse(Path::new(sk))?;

    tracing::info!("Updating extra kanji");
    update_kanji_extra(&kd2, &jmdict, &mut ke, ke_path, &sk)?;

    conn.transaction(|conn| {
        tracing::info!("Starting transaction");
        update_kanji(conn, &kd2, &ke, &kf, &kn, &ks, &sk).context("Failed to update kanji")?;
        // update_words(conn, &jmdict).context("Failed to update words")?;
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
    sk: &HashMap<String, Vec<String>>,
) -> eyre::Result<()> {
    let kanjidic_kanji = kd2.character.iter().map(|c| c.literal.as_str());
    let extra_kanji = ke.kanji_extra.iter().map(|ke| ke.chara.as_str());
    let included_kanji = kanjidic_kanji.chain(extra_kanji).collect::<HashSet<_>>();

    let jmdict_kanji = jmdict
        .entry
        .iter()
        .flat_map(|e| &e.k_ele)
        .map(|kele| &kele.keb)
        .flat_map(|keb| lbr::kanji_from_word(keb));
    let similar_kanji = sk
        .keys()
        .map(String::as_str)
        .chain(sk.values().flatten().map(String::as_str));
    let external_kanji = jmdict_kanji.chain(similar_kanji).collect::<HashSet<_>>();

    let missing_kanji = external_kanji
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
        ke.kanji_extra
            .sort_unstable_by(|a, b| a.chara.cmp(&b.chara));
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
    sk: &HashMap<String, Vec<String>>,
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
        tracing::debug!("Processing kanji {}", kanji.literal);

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
            .map(|m| &m.text)
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
            .filter(|r| r.r_type == "ja_on" || r.r_type == "ja_kun")
        {
            let (reading, okurigana) = reading
                .text
                .split_once(".")
                .map(|(r, o)| (r, Some(o)))
                .unwrap_or((&reading.text, None));
            new_kanji_readings.push((
                kr::kanji_id.eq(kanji_id),
                kr::reading.eq(reading),
                kr::okurigana.eq(okurigana),
                kr::nanori.eq(false),
            ));
        }
        for nanori in kanji
            .reading_meaning
            .as_ref()
            .map(|rm| rm.nanori.as_slice())
            .unwrap_or_default()
        {
            new_kanji_readings.push((
                kr::kanji_id.eq(kanji_id),
                kr::reading.eq(nanori.as_str()),
                kr::okurigana.eq(None),
                kr::nanori.eq(true),
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
    for (kanji, similar) in ks.kanji_similar.iter().chain(sk.iter()) {
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
        diesel::insert_into(ks::table)
            .values(chunk)
            .on_conflict_do_nothing()
            .execute(conn)?;
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

fn update_words(conn: &mut PgConnection, jmdict: &JMdict) -> eyre::Result<()> {
    use lbr_server::schema::{kanji as k, word_kanji as wk, words as w};

    let kanji_to_readings = domain::japanese::kanji_to_readings(conn)?;
    std::fs::write("./data/kanjimap", format!("{kanji_to_readings:#?}"))?;

    tracing::info!("Updating words");
    let existing_words_vec = w::table
        .select((w::jmdict_id, w::id, w::word, w::reading_standard))
        .get_results::<(i32, i32, String, String)>(conn)?;
    // (jmdict_id, word, standardised reading) => id
    let existing_words = existing_words_vec
        .iter()
        .map(|(jmdict_id, id, word, reading)| ((*jmdict_id, word.as_str(), reading.as_str()), *id))
        .collect::<HashMap<(i32, &str, &str), i32>>();
    let kanji_map = k::table
        .select((k::chara, k::id))
        .get_results::<(String, i32)>(conn)?
        .into_iter()
        .collect::<HashMap<_, _>>();

    let mut words_in_db = HashSet::new();
    for entry in &jmdict.entry {
        let jmdict_id = entry
            .ent_seq
            .parse::<i32>()
            .context("Failed to parse jmdict seq")?;
        tracing::trace!("Processing entry {}", jmdict_id);

        // previously we did more filtering here such as excluding search-only kanji forms
        // but since they actually appear in text we should include all of them
        let mut jmdict_words = JmdictWords::new();
        for k_ele in &entry.k_ele {
            // only include reading elements that do not exclude the kanji element
            let valid_reading_elements = entry
                .r_ele
                .iter()
                .filter(|r_ele| r_ele.re_restr.is_empty() || r_ele.re_restr.contains(&k_ele.keb))
                .collect::<Vec<_>>();
            if valid_reading_elements.is_empty() {
                // some kanji elements like search-only kanji forms do not have any valid reading element associated with them...
                // in those cases, we will simply try out every single one and accept any that have a valid furigana mapping
                for r_ele in &entry.r_ele {
                    if !furigana::map(&k_ele.keb, &r_ele.reb, &kanji_to_readings).is_empty() {
                        let word = &k_ele.keb;
                        let reading = &r_ele.reb;
                        let translations = translations_from_entry(entry, Some(word), reading);
                        jmdict_words.insert_new(jmdict_id, word, reading, translations);
                    }
                }
            }
            for r_ele in valid_reading_elements {
                let word = &k_ele.keb;
                let reading = &r_ele.reb;
                let translations = translations_from_entry(entry, Some(word), reading);
                jmdict_words.insert_new(jmdict_id, word, reading, translations);
            }
        }

        // adds all reading elements as their own words as well
        // previously this was only done for words with no kanji elements,
        // but there are too many words like 時 that are often spelled like とき with kana
        // with no indication of this in JMdict. although doing this will result in many words
        // that might not be considered real because they are in reality never spelled using kana,
        // their existence in the db shouldn't be harmful. if needed we can later tag these "extra" words
        // in the db so we can tell them apart from the ones that exist in JMdict
        for r_ele in &entry.r_ele {
            let word = &r_ele.reb;
            let reading = &r_ele.reb;
            let translations = translations_from_entry(entry, None, reading);
            jmdict_words.insert_new(jmdict_id, word, reading, translations);
        }

        for (key, val) in jmdict_words.jmdict_words {
            let JmdictWordKey {
                jmdict_id,
                word,
                standardised_reading,
            } = key;
            let JmdictWordVal {
                reading,
                hiragana_reading,
                translations,
            } = val;
            let furigana =
                match domain::japanese::map_to_db_furigana(&word, &reading, &kanji_to_readings) {
                    Ok(furigana) => furigana,
                    Err(err) => {
                        tracing::error!("Failed to map furigana {err}");
                        Vec::new()
                    }
                };
            let existing_word = existing_words
                .get(&(jmdict_id, &word, &standardised_reading))
                .copied();
            if let Some(id) = existing_word {
                // update existing record
                diesel::update(w::table.filter(w::id.eq(id)))
                    .set((w::translations.eq(translations), w::furigana.eq(furigana)))
                    .execute(conn)?;
                words_in_db.insert(id);
            } else {
                tracing::debug!("Creating {jmdict_id} {word} ({hiragana_reading})");
                // create new
                let word_id = diesel::insert_into(w::table)
                    .values((
                        w::jmdict_id.eq(jmdict_id),
                        w::word.eq(&word),
                        w::reading.eq(&hiragana_reading),
                        w::reading_standard.eq(&standardised_reading),
                        w::furigana.eq(&furigana),
                        w::translations.eq(&translations),
                    ))
                    .returning(w::id)
                    .get_result::<i32>(conn)
                    .context("Failed to create new word")?;
                // for new words, we also insert the word kanji
                let mut word_kanji = Vec::new();
                for kanji in lbr::kanji_from_word(&word) {
                    let kanji_id = kanji_map
                        .get(kanji)
                        .copied()
                        .wrap_err_with(|| format!("Failed to find kanji {kanji}"))?;
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
    let existing_word_ids = existing_words_vec
        .iter()
        .map(|a| (a.1, a))
        .collect::<HashMap<_, _>>();
    for (existing_word_id, etc) in existing_word_ids {
        if !words_in_db.contains(&existing_word_id) {
            tracing::error!(
                "word in db {existing_word_id} {} {} not found while processing jmdict",
                etc.2,
                etc.3
            );
            diesel::delete(wk::table)
                .filter(wk::word_id.eq(existing_word_id))
                .execute(conn)?;
            diesel::delete(w::table)
                .filter(w::id.eq(existing_word_id))
                .execute(conn)?;
        }
    }

    Ok(())
}

#[derive(Hash, PartialEq, Eq)]
struct JmdictWordKey {
    jmdict_id: i32,
    word: String,
    standardised_reading: String,
}

#[derive(Hash, PartialEq, Eq)]
struct JmdictWordVal {
    reading: String,
    hiragana_reading: String,
    translations: Vec<String>,
}

struct JmdictWords {
    jmdict_words: HashMap<JmdictWordKey, JmdictWordVal>,
}

impl JmdictWords {
    fn new() -> Self {
        Self {
            jmdict_words: HashMap::new(),
        }
    }

    fn insert_new(&mut self, jmdict_id: i32, word: &str, reading: &str, translations: Vec<String>) {
        let word = word.to_owned();
        let reading = reading.to_owned();
        let sr = lbr::standardise_reading(&reading);
        self.jmdict_words.insert(
            JmdictWordKey {
                jmdict_id,
                word,
                standardised_reading: sr.standardised,
            },
            JmdictWordVal {
                reading,
                hiragana_reading: sr.hiragana,
                translations,
            },
        );
    }
}

fn translations_from_entry(entry: &Entry, word: Option<&str>, reading: &str) -> Vec<String> {
    entry
        .sense
        .iter()
        .filter(|s| {
            // with no word, we can ignore stagk
            s.stagk.is_empty()
                || word
                    .map(|w| s.stagk.iter().any(|stagk| stagk == w))
                    .unwrap_or(true)
        })
        .filter(|s| s.stagr.is_empty() || s.stagr.iter().any(|stagr| stagr == reading))
        .map(|s| {
            let sense_meanings = s
                .gloss
                .iter()
                .filter(|g| g.lang.is_none())
                .map(|g| g.text.as_str())
                .collect::<Vec<_>>()
                .join(", ");

            if s.s_inf.is_empty() {
                sense_meanings
            } else {
                format!("{} [{}]", sense_meanings, s.s_inf.join(", "))
            }
        })
        .collect::<Vec<_>>()
}
