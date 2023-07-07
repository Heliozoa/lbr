//! Types and functionality for parsing and creating a wordfile.

use crate::{
    jmdict::{JMDict, REle, Sense},
    jmdict_furigana,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};
use wana_kana::ConvertJapanese;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wordfile {
    pub header: Header,
    pub words: Vec<Word>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub source_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub jmdict_id: i32,
    pub word: Vec<Form>,
    pub meanings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Form {
    pub written_form: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub readings: Vec<Reading>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reading {
    pub reading: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub furigana: Vec<Furigana>,
    #[serde(default)]
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub usually_kana: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Furigana {
    pub start_idx: usize,
    pub end_idx: usize,
    pub furigana: String,
}

impl Wordfile {
    pub fn from_jmdict_with_furigana(
        jmdict: JMDict,
        jmdict_version: String,
        furigana: Vec<jmdict_furigana::Furigana>,
    ) -> Self {
        let header = Header {
            source_version: jmdict_version,
        };

        let furigana = process_furigana(furigana);
        let tuples = into_tuples(jmdict, &furigana);
        let words = tuples_into_words(tuples);

        Wordfile { header, words }
    }
}

fn process_furigana(
    furigana: Vec<jmdict_furigana::Furigana>,
) -> HashMap<(String, String), Vec<Furigana>> {
    furigana
        .into_iter()
        .map(|f| {
            let key = (f.text, f.reading);
            let mut furigana = vec![];
            let mut start_idx = 0;
            for ruby in f.furigana {
                let end_idx = start_idx + ruby.ruby.len();
                if let Some(rt) = ruby.rt {
                    furigana.push(Furigana {
                        start_idx,
                        end_idx,
                        furigana: rt,
                    });
                }
                start_idx = end_idx;
            }
            (key, furigana)
        })
        .collect()
}

#[derive(Debug)]
struct Tuple {
    jmdict_id: i32,
    written_form: String,
    reading: Option<String>,
    furigana: Vec<Furigana>,
    meanings: Vec<String>,
    usually_kana: bool,
}

fn into_tuples(jmdict: JMDict, furigana: &HashMap<(String, String), Vec<Furigana>>) -> Vec<Tuple> {
    let mut tuples = vec![];
    for entry in jmdict.entry {
        if entry.k_ele.is_empty() {
            for rele in &entry.r_ele {
                tuples.push(reading_into_tuples(
                    entry.ent_seq.parse().expect("invalid id"),
                    furigana,
                    &entry.sense,
                    None,
                    rele,
                    false,
                ));
            }
        } else {
            for kele in entry.k_ele {
                let rare_written_form = kele.ke_inf.iter().any(|s| s == "rarely-used kanji form");
                let keb = kele.keb;
                for rele in &entry.r_ele {
                    if rele.re_restr.is_empty() || rele.re_restr.contains(&keb) {
                        tuples.push(reading_into_tuples(
                            entry.ent_seq.parse().expect("invalid id"),
                            furigana,
                            &entry.sense,
                            Some(keb.clone()),
                            rele,
                            rare_written_form,
                        ));
                    }
                }
            }
        }
    }
    tuples
}

fn reading_into_tuples(
    jmdict_id: i32,
    furigana: &HashMap<(String, String), Vec<Furigana>>,
    sense: &[Sense],
    keb: Option<String>,
    rele: &REle,
    rare_written_form: bool,
) -> Tuple {
    let reb = rele.reb.clone();
    let keb = keb.unwrap_or_else(|| reb.clone());
    let tuple = (keb.clone(), reb.clone());
    let furigana = furigana.get(&tuple).cloned().unwrap_or_default();
    let mut usually_kana = rare_written_form;
    let mut meanings = vec![];
    for s in sense {
        if s.misc
            .iter()
            .any(|m| m == "word usually written using kana alone")
        {
            usually_kana = true;
        }
        let stagk = s.stagk.is_empty() || s.stagk.contains(&keb);
        let stagr = s.stagr.is_empty() || s.stagr.contains(&reb);
        if stagk && stagr {
            for g in s.gloss.iter().filter_map(|g| {
                if g.lang.is_none() {
                    Some(g.value.clone())
                } else {
                    None
                }
            }) {
                meanings.push(g);
            }
        }
    }
    Tuple {
        jmdict_id,
        written_form: keb.clone(),
        reading: if keb == reb { None } else { Some(reb) },
        furigana,
        meanings,
        usually_kana,
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct WordKey {
    written_form_katakana: String,
    meanings_ordered: Vec<String>,
}

impl WordKey {
    fn from_tuple(tuple: &Tuple) -> Self {
        let mut meanings_ordered = tuple.meanings.clone();
        meanings_ordered.sort();
        let wfh = tuple.written_form.to_katakana();
        Self {
            written_form_katakana: wfh,
            meanings_ordered,
        }
    }
}

fn tuples_into_words(tuples: Vec<Tuple>) -> Vec<Word> {
    let mut map: HashMap<WordKey, Word> = HashMap::new();
    for tuple in tuples {
        let key = WordKey::from_tuple(&tuple);
        let form = Form {
            written_form: tuple.written_form,
            readings: tuple
                .reading
                .map(|reading| {
                    vec![Reading {
                        reading,
                        furigana: tuple.furigana,
                        usually_kana: tuple.usually_kana,
                    }]
                })
                .unwrap_or_default(),
        };
        match map.entry(key) {
            Entry::Occupied(mut occupied) => {
                let occupied = occupied.get_mut();
                match occupied
                    .word
                    .iter_mut()
                    .find(|f| f.written_form == form.written_form)
                {
                    Some(matching_form) => matching_form.readings.extend(form.readings),
                    None => occupied.word.push(form),
                }
            }
            Entry::Vacant(vacant) => {
                vacant.insert(Word {
                    jmdict_id: tuple.jmdict_id,
                    word: vec![form],
                    meanings: tuple.meanings,
                });
            }
        }
    }
    let mut vals = map.into_values().collect::<Vec<_>>();
    for val in &mut vals {
        val.word.sort_by(|l, r| l.written_form.cmp(&r.written_form));
    }
    vals.sort_by(|l, r| {
        l.word
            .first()
            .map(|f| &f.written_form)
            .cmp(&r.word.first().map(|f| &f.written_form))
    });
    vals
}
