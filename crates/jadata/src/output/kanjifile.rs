//! Types and functionality for parsing and creating a kanjifile.

use crate::{
    kanjidic2::{self, Character, Kanjidic2},
    kanjifile_manual::KanjifileManual,
    kanjifile_names::KanjifileNames,
    kanjifiles_similar::KanjifileSimilar,
    kradfile::Kradfile,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kanjifile {
    pub header: Header,
    pub kanji: Vec<Kanji>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub source_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kanji {
    pub kanji: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub readings: Vec<Reading>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub meanings: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub similar: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reading {
    pub reading: String,
    pub kind: ReadingKind,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub okurigana: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Position {
    Prefix,
    Suffix,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ReadingKind {
    Onyomi,
    Kunyomi,
}

impl Kanjifile {
    pub fn derive(
        kd2: Kanjidic2,
        kf: Kradfile,
        kfn: KanjifileNames,
        kfs: KanjifileSimilar,
        kfm: KanjifileManual,
    ) -> Self {
        let kanji_names = kfn
            .kanji
            .into_iter()
            .map(|kn| (kn.kanji, kn.name))
            .collect::<HashMap<_, _>>();
        let similar_kanji = kfs
            .kanji
            .into_iter()
            .map(|sk| (sk.kanji, sk.similar))
            .collect::<HashMap<_, _>>();

        let mut kanji_list = kfm.kanji;
        let mut seen = HashSet::new();
        for kanji in kd2.character {
            if kanji.literal.chars().count() != 1 {
                panic!("multi-codepoint literal");
            }
            if !seen.insert(kanji.literal.clone()) {
                panic!("repeated kanji");
            }
            let kanji = handle_kanji(kanji, &kanji_names, &similar_kanji, &kf.kanji_to_components);
            kanji_list.push(kanji)
        }
        kanji_list.sort_by(|l, r| l.kanji.cmp(&r.kanji));

        let header = Header {
            source_version: kd2.header.file_version,
        };
        Kanjifile {
            header,
            kanji: kanji_list,
        }
    }
}

fn handle_kanji(
    kanji: Character,
    kanji_names: &HashMap<String, String>,
    similar_kanji: &HashMap<String, Vec<String>>,
    kanji_to_components: &HashMap<String, Vec<String>>,
) -> Kanji {
    let mut meanings = vec![];
    let mut readings = vec![];
    for rmg in kanji.reading_meaning.into_iter().flat_map(|rm| rm.rmgroup) {
        meanings.extend(handle_meanings(rmg.meaning));
        readings.extend(handle_readings(rmg.reading));
    }
    meanings.sort();
    readings.sort_by(|l, r| l.reading.cmp(&r.reading));

    // use first meaning if no name set
    let name = kanji_names
        .get(&kanji.literal)
        .or_else(|| meanings.first())
        .cloned();
    let similar = similar_kanji
        .get(&kanji.literal)
        .cloned()
        .unwrap_or_default();

    Kanji {
        components: kanji_to_components
            .get(&kanji.literal)
            .cloned()
            .unwrap_or_default(),
        kanji: kanji.literal,
        name,
        meanings,
        readings,
        similar,
    }
}

fn handle_meanings(meanings: Vec<kanjidic2::Meaning>) -> impl Iterator<Item = String> {
    meanings
        .into_iter()
        .filter(|m| m.m_lang.is_none())
        .map(|m| m.value)
}

fn handle_readings(readings: Vec<kanjidic2::Reading>) -> impl Iterator<Item = Reading> {
    readings.into_iter().filter_map(|r| {
        let kind = match r.r_type.as_str() {
            "ja_on" => ReadingKind::Onyomi,
            "ja_kun" => ReadingKind::Kunyomi,
            _ => return None,
        };
        let position = if r.value.starts_with('-') {
            Some(Position::Suffix)
        } else if r.value.ends_with('-') {
            Some(Position::Prefix)
        } else {
            None
        };
        let (reading, okurigana) = if let Some((reading, okurigana)) = r.value.split_once('.') {
            (
                reading.trim_matches('-').to_string(),
                Some(okurigana.trim_matches('-').to_string()),
            )
        } else {
            (r.value.trim_matches('-').to_string(), None)
        };
        Some(Reading {
            kind,
            reading,
            okurigana,
            position,
        })
    })
}
