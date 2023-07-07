//! Functions and types related to LBR decks.

use crate::{eq, utils::database};
use diesel::prelude::*;
use itertools::Itertools;
use lbr::{
    anki,
    anki::{Card, Deck, Kanji, Sentence, SentenceWord},
};
use rand::seq::SliceRandom;
use std::collections::HashMap;

/// Generates an Anki deck for the given deck id.
pub fn gen_deck(
    conn: &mut PgConnection,
    name: &str,
    deck_id: i32,
    anki_deck_id: i64,
) -> eyre::Result<Deck> {
    tracing::info!("Fetching words");
    let cards = get_cards(conn, deck_id)?;

    tracing::info!("Creating deck");
    let deck = lbr::anki::create_deck(name, anki_deck_id, cards);
    Ok(deck)
}

crate::query! {
    #[derive(Debug, Clone)]
    struct KanjiQuery {
        kanji: String = kanji::chara,
        name: Option<String> = kanji::name,
    }
}

crate::query! {
    #[derive(Debug, Clone)]
    struct SentenceWordQuery {
        word_id: i32 = words::id,
        sentence: String = sentences::sentence,
        idx_start: i32 = sentence_words::idx_start,
        idx_end: i32 = sentence_words::idx_end,
        reading: Option<String> = sentence_words::reading,
        // postgres doesn't support non-null constraints on array elements,
        // so these are Options even though they're never None
        furigana: Vec<Option<database::Furigana>> = sentence_words::furigana,
        // postgres doesn't support non-null constraints on array elements,
        // so these are Options even though they're never None
        translations: Vec<Option<String>> = words::translations,
        sentence_id: i32 = sentences::id,
    }
}

fn get_cards(conn: &mut PgConnection, deck_id: i32) -> eyre::Result<Vec<Card>> {
    use crate::schema::{
        deck_sources as ds, kanji as k, sentence_words as sw, sentences as s, word_kanji as wk,
        words as w, written_forms as wf,
    };

    // get all sentence words for the deck
    let sentence_words: Vec<SentenceWordQuery> = ds::table
        .filter(eq!(ds, deck_id))
        // get all sentences for the deck's sources
        .inner_join(s::table.on(s::source_id.eq(ds::source_id)))
        // get all words related to the sentences
        .inner_join(sw::table.on(sw::sentence_id.eq(s::id)))
        .inner_join(w::table.on(w::id.eq(sw::word_id)))
        .select(SentenceWordQuery::as_select())
        .load(conn)?;

    let sentence_word_word_ids = sentence_words
        .iter()
        .map(|sw| sw.word_id)
        .collect::<Vec<_>>();
    // get all kanji related to the sentences
    let kanji: Vec<KanjiQuery> = w::table
        // get all words related to the sentences
        .filter(w::id.eq_any(sentence_word_word_ids))
        .inner_join(wf::table.on(wf::word_id.eq(w::id)))
        // get all kanji related to the words
        .inner_join(wk::table.on(wk::written_form_id.eq(wf::id)))
        .inner_join(k::table.on(k::id.eq(wk::kanji_id)))
        .select(KanjiQuery::as_select())
        .load(conn)?;
    let kanji_names_by_kanji = kanji
        .into_iter()
        .map(|k| (k.kanji, k.name))
        .collect::<HashMap<_, _>>();
    let sentence_words_by_word_id = sentence_words
        .iter()
        .cloned()
        .into_group_map_by(|r| r.word_id);
    let sentence_words_by_sentence = sentence_words
        .into_iter()
        .into_group_map_by(|r| r.sentence_id);

    // for each word, randomly select one sentence
    let mut cards = vec![];
    for (_, rows) in sentence_words_by_word_id {
        // for each word, choose random sentence word
        let query = rows.choose(&mut rand::thread_rng()).cloned().unwrap();

        let card = card_from_query(
            query,
            &sentence_words_by_sentence,
            &kanji_names_by_kanji,
            rows.len(),
        );
        cards.push(card);
    }
    Ok(cards)
}

fn card_from_query(
    query: SentenceWordQuery,
    sentence_words_by_sentence: &HashMap<i32, Vec<SentenceWordQuery>>,
    kanji_names_by_kanji: &HashMap<String, Option<String>>,
    rows: usize,
) -> Card {
    let SentenceWordQuery {
        word_id,
        sentence,
        idx_start,
        idx_end,
        reading,
        furigana,
        translations,
        sentence_id,
    } = query;

    let word_in_sentence = &sentence[idx_start as usize..idx_end as usize];
    let sentence_words = sentence_words_by_sentence.get(&sentence_id).unwrap();
    let kanji = lbr::kanji_from_word(word_in_sentence)
        .map(|k| Kanji {
            name: kanji_names_by_kanji
                .get(k)
                .and_then(Option::as_ref)
                .cloned(),
            chara: k.to_string(),
            related_word: None, // TODO
        })
        .collect();

    Card {
        id: word_id,
        word: " ".to_string(),
        word_range: idx_start as usize..idx_end as usize,
        word_furigana: db_furigana_to_anki_furigana(furigana.as_slice(), reading.as_deref()),
        translations: translations.into_iter().flatten().collect(),
        kanji,
        sentence: Sentence {
            sentence,
            words: sentence_words
                .iter()
                .map(|r| SentenceWord {
                    furigana: db_furigana_to_anki_furigana(
                        r.furigana.as_slice(),
                        r.reading.as_deref(),
                    ),
                    idx_start: r.idx_start,
                    idx_end: r.idx_end,
                })
                .collect(),
            sentence_count: rows,
        },
    }
}

fn db_furigana_to_anki_furigana(
    furigana: &[Option<database::Furigana>],
    reading: Option<&str>,
) -> Vec<anki::Furigana> {
    furigana
        .iter()
        .flatten()
        .filter_map(|f| {
            reading.as_ref().map(|reading| anki::Furigana {
                range: f.word_start_idx as usize..f.word_end_idx as usize,
                furigana: reading[f.reading_start_idx as usize..f.reading_end_idx as usize]
                    .to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::database::Furigana as DbFurigana;
    use lbr::anki::Furigana as AnkiFurigana;

    #[test]
    fn makes_card_from_query() {
        let word_id = 1;
        let sentence_id = 2;
        let query = SentenceWordQuery {
            word_id,
            sentence: "吾輩は猫である".to_string(),
            idx_start: 9,
            idx_end: 12,
            reading: Some("ねこ".to_string()),
            furigana: vec![Some(DbFurigana {
                word_start_idx: 0,
                word_end_idx: 3,
                reading_start_idx: 0,
                reading_end_idx: 6,
            })],
            translations: vec![Some("Cat".to_string())],
            sentence_id,
        };
        let mut sentence_words_by_sentence = HashMap::new();
        sentence_words_by_sentence.insert(
            sentence_id,
            vec![
                SentenceWordQuery {
                    word_id,
                    sentence: "吾輩は猫である".to_string(),
                    idx_start: 0,
                    idx_end: 6,
                    reading: Some("わがはい".to_string()),
                    furigana: vec![
                        Some(DbFurigana {
                            word_start_idx: 0,
                            word_end_idx: 3,
                            reading_start_idx: 0,
                            reading_end_idx: 6,
                        }),
                        Some(DbFurigana {
                            word_start_idx: 3,
                            word_end_idx: 6,
                            reading_start_idx: 6,
                            reading_end_idx: 12,
                        }),
                    ],
                    translations: vec![Some("I".to_string())],
                    sentence_id,
                },
                SentenceWordQuery {
                    word_id,
                    sentence: "吾輩は猫である".to_string(),
                    idx_start: 6,
                    idx_end: 9,
                    reading: None,
                    furigana: vec![],
                    translations: vec![Some("tldr".to_string())],
                    sentence_id,
                },
                SentenceWordQuery {
                    word_id,
                    sentence: "吾輩は猫である".to_string(),
                    idx_start: 9,
                    idx_end: 12,
                    reading: Some("ねこ".to_string()),
                    furigana: vec![Some(DbFurigana {
                        word_start_idx: 0,
                        word_end_idx: 3,
                        reading_start_idx: 0,
                        reading_end_idx: 6,
                    })],
                    translations: vec![Some("Cat".to_string())],
                    sentence_id,
                },
                SentenceWordQuery {
                    word_id,
                    sentence: "吾輩は猫である".to_string(),
                    idx_start: 12,
                    idx_end: 15,
                    reading: None,
                    furigana: vec![],
                    translations: vec![Some("something".to_string())],
                    sentence_id,
                },
                SentenceWordQuery {
                    word_id,
                    sentence: "吾輩は猫である".to_string(),
                    idx_start: 15,
                    idx_end: 18,
                    reading: None,
                    furigana: vec![],
                    translations: vec![],
                    sentence_id,
                },
            ],
        );

        let mut kanji_names_by_kanji = HashMap::new();
        kanji_names_by_kanji.insert("猫".to_string(), Some("cat".to_string()));

        let card = card_from_query(query, &sentence_words_by_sentence, &kanji_names_by_kanji, 1);
        assert_eq!(card.sentence.words[0].furigana[0].furigana, "わが");
    }

    #[test]
    fn converts_db_furigana_to_anki_furigana() {
        let furigana = &[
            Some(DbFurigana {
                word_start_idx: 0,
                word_end_idx: 3,
                reading_start_idx: 0,
                reading_end_idx: 3,
            }),
            Some(DbFurigana {
                word_start_idx: 3,
                word_end_idx: 6,
                reading_start_idx: 3,
                reading_end_idx: 6,
            }),
        ];
        let furigana = db_furigana_to_anki_furigana(furigana, Some("こや"));
        assert_eq!(furigana.len(), 2, "{furigana:#?}");
        assert_eq!(
            furigana[0],
            AnkiFurigana {
                range: 0..3,
                furigana: "こ".to_string()
            },
            "{:#?}",
            furigana[0]
        );
        assert_eq!(
            furigana[1],
            AnkiFurigana {
                range: 3..6,
                furigana: "や".to_string()
            },
            "{:#?}",
            furigana[1]
        );
    }
}
