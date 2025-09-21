//! Functions and types related to LBR decks.

use crate::utils::database::{self, DeckSourceKind};
use diesel::prelude::*;
use itertools::Itertools;
use lbr::anki::{self, Deck, KanjiCard, Sentence, SentenceWord, WordCard, WordKanji};
use rand::seq::IndexedRandom;
use std::collections::{HashMap, HashSet};

/// Generates an Anki deck for the given deck id.
pub fn gen_deck(
    conn: &mut PgConnection,
    name: String,
    deck_id: i32,
    anki_deck_id: i64,
    user_id: i32,
) -> eyre::Result<Deck> {
    tracing::info!("Creating cards");
    let word_cards = get_word_cards(conn, user_id, deck_id)?;
    let kanji_cards = get_kanji_cards(conn, deck_id)?;
    tracing::debug!("Created {} cards", word_cards.len() + kanji_cards.len());

    tracing::info!("Creating deck");
    let package = lbr::anki::create_deck(name, anki_deck_id, word_cards, kanji_cards);
    tracing::info!("Created deck");

    Ok(package)
}

fn get_word_cards(
    conn: &mut PgConnection,
    user_id: i32,
    deck_id: i32,
) -> eyre::Result<Vec<WordCard>> {
    use crate::schema::{
        deck_sources as ds, ignored_words as iw, kanji as k, sentence_words as sw, sentences as s,
        word_kanji as wk, words as w,
    };

    let ignored_words = iw::table
        .filter(iw::user_id.eq(user_id))
        .select(iw::word_id)
        .get_results::<i32>(conn)?
        .into_iter()
        .collect::<HashSet<_>>();

    // get all sentence words for the deck
    let sentence_words: Vec<SentenceWordQuery> = ds::table
        // get all word sources for the deck
        .filter(
            ds::deck_id
                .eq(deck_id)
                .and(ds::kind.eq(DeckSourceKind::Word)),
        )
        // get all sentences for the deck's sources
        .inner_join(s::table.on(s::source_id.eq(ds::source_id)))
        // get all words related to the sentences
        .inner_join(sw::table.on(sw::sentence_id.eq(s::id)))
        // left joins because some sentence words are not associated with any word (reading only)
        .left_join(w::table.on(w::id.nullable().eq(sw::word_id)))
        .select(SentenceWordQuery::as_select())
        .load(conn)?;

    let sentence_word_word_ids = sentence_words
        .iter()
        .filter_map(|sw| sw.word_id)
        .filter(|wi| !ignored_words.contains(wi))
        .collect::<Vec<_>>();
    // get all kanji related to the sentences
    let kanji: Vec<KanjiQuery> = w::table
        // get all words related to the sentences
        .filter(w::id.eq_any(sentence_word_word_ids))
        // get all kanji related to the words
        .inner_join(wk::table.on(wk::word_id.eq(w::id)))
        .inner_join(k::table.on(k::id.eq(wk::kanji_id)))
        .select(KanjiQuery::as_select())
        .load(conn)?;
    let kanji_names_by_kanji = kanji
        .into_iter()
        .map(|k| (k.kanji, k.name))
        .collect::<HashMap<_, _>>();
    let sentence_words_by_word_id = sentence_words
        .iter()
        .filter_map(|sw| sw.word_id.map(|wi| (wi, sw)))
        .into_group_map();
    let sentence_words_by_sentence = sentence_words.iter().into_group_map_by(|r| r.sentence_id);

    let mut cards = Vec::new();
    for (_, word_sentences) in sentence_words_by_word_id {
        // for each word, choose random sentence
        let sentence = word_sentences.choose(&mut rand::rng()).cloned().unwrap();
        let sentence_words = sentence_words_by_sentence
            .get(&sentence.sentence_id)
            .unwrap();

        let card = word_card_from_query(
            sentence,
            sentence_words,
            &kanji_names_by_kanji,
            word_sentences.len(),
        );
        cards.push(card);
    }
    Ok(cards)
}

fn get_kanji_cards(conn: &mut PgConnection, deck_id: i32) -> eyre::Result<Vec<KanjiCard>> {
    use crate::schema::{
        deck_sources as ds, kanji as k, kanji_similar as ks, sentence_words as sw, sentences as s,
        word_kanji as wk, words as w,
    };

    // get all words from kanji sources for the deck
    let kind = DeckSourceKind::Kanji;
    let kanji_words: Vec<KanjiWordQuery> = ds::table
        .filter(ds::deck_id.eq(deck_id).and(ds::kind.eq(kind)))
        // get all sentences for the deck's sources
        .inner_join(s::table.on(s::source_id.eq(ds::source_id)))
        // get all words related to the sentences
        .inner_join(sw::table.on(sw::sentence_id.eq(s::id)))
        .inner_join(w::table.on(w::id.nullable().eq(sw::word_id)))
        // get all kanji related to the words
        .inner_join(wk::table.on(wk::word_id.eq(w::id)))
        .inner_join(k::table.on(k::id.eq(wk::kanji_id)))
        .filter(k::name.is_not_null())
        .select(KanjiWordQuery::as_select())
        .load(conn)?;
    let kanji_ids = kanji_words
        .iter()
        .map(|kw| kw.kanji_id)
        .collect::<HashSet<_>>();

    if kanji_ids.contains(&983) {
        panic!("uh oh");
    }

    let source_words_by_kanji_id: HashMap<i32, Vec<KanjiWordQuery>> = kanji_words
        .iter()
        .cloned()
        .into_group_map_by(|swq| swq.kanji_id);
    let similar_kanji_lower: Vec<SimilarKanjiQuery> = ks::table
        .inner_join(k::table.on(k::id.eq(ks::lower_kanji_id)))
        .select(SimilarKanjiQuery::as_select())
        .load(conn)?;
    let similar_kanji_higher: Vec<SimilarKanjiQuery> = ks::table
        .inner_join(k::table.on(k::id.eq(ks::higher_kanji_id)))
        .select(SimilarKanjiQuery::as_select())
        .load(conn)?;
    let mut higher_kanji_id_to_similar_kanji = similar_kanji_lower
        .into_iter()
        .into_group_map_by(|skq| skq.higher_kanji_id);
    let mut lower_kanji_id_to_similar_kanji = similar_kanji_higher
        .into_iter()
        .into_group_map_by(|skq| skq.lower_kanji_id);

    let mut cards = Vec::new();
    for (kanji_id, words) in source_words_by_kanji_id {
        // for each kanji, choose random example word
        let word = words.choose(&mut rand::rng()).cloned().unwrap();
        let mut higher_similar_kanji = lower_kanji_id_to_similar_kanji
            .remove(&kanji_id)
            .unwrap_or_default();
        let mut lower_similar_kanji = higher_kanji_id_to_similar_kanji
            .remove(&kanji_id)
            .unwrap_or_default();
        higher_similar_kanji.retain(|sk| kanji_ids.contains(&sk.higher_kanji_id));
        lower_similar_kanji.retain(|sk| kanji_ids.contains(&sk.lower_kanji_id));
        higher_similar_kanji.extend(lower_similar_kanji.into_iter());
        let card = kanji_card_from_query(word, higher_similar_kanji, words.len());
        cards.push(card);
    }
    Ok(cards)
}

fn word_card_from_query(
    word: &SentenceWordQuery,
    sentence_words: &[&SentenceWordQuery],
    kanji_names_by_kanji: &HashMap<String, Option<String>>,
    word_sentences: usize,
) -> WordCard {
    let SentenceWordQuery {
        word_id,
        word,
        reading,
        sentence,
        sentence_word_reading: _,
        sentence_word_furigana: _,
        idx_start,
        idx_end,
        furigana,
        translations,
        sentence_id,
    } = word.clone();

    let word_in_sentence = &sentence[idx_start as usize..idx_end as usize];
    let kanji = lbr::kanji_from_word(word_in_sentence)
        .map(|k| WordKanji {
            name: kanji_names_by_kanji
                .get(k)
                .and_then(Option::as_ref)
                .cloned(),
            chara: k.to_string(),
        })
        .collect();

    WordCard {
        id: word_id.unwrap(),
        word_id: word_id.unwrap(),
        word: word.unwrap(),
        word_range: idx_start as usize..idx_end as usize,
        word_furigana: db_furigana_to_anki_furigana(
            furigana.unwrap().as_slice(),
            Some(&reading.unwrap()),
        ),
        translations: translations.unwrap().into_iter().flatten().collect(),
        kanji,
        word_sentences,
        sentence: Sentence {
            id: sentence_id,
            sentence,
            words: sentence_words
                .iter()
                .map(|r| SentenceWord {
                    furigana: db_furigana_to_anki_furigana(
                        r.sentence_word_furigana.as_slice(),
                        r.sentence_word_reading.as_deref(),
                    ),
                    idx_start: r.idx_start,
                    idx_end: r.idx_end,
                })
                .collect(),
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

fn kanji_card_from_query(
    kanji: KanjiWordQuery,
    similar_kanji: Vec<SimilarKanjiQuery>,
    word_count: usize,
) -> KanjiCard {
    let similar_kanji = similar_kanji
        .into_iter()
        .map(|skq| anki::Kanji {
            kanji: skq.kanji,
            name: skq.name,
        })
        .collect();
    KanjiCard {
        id: kanji.kanji_id,
        kanji: kanji.kanji,
        name: kanji.kanji_name.unwrap_or_default(),
        example_source_word: anki::KanjiWord {
            word: kanji.written_form,
            translations: kanji.translations.into_iter().flatten().collect(),
        },
        similar_kanji,
        kanji_words: word_count,
    }
}

// queries

crate::query! {
    #[derive(Debug, Clone)]
    struct KanjiQuery {
        kanji: String = kanji::chara,
        name: Option<String> = kanji::name,
    }
}

crate::newquery! {
    #[derive(Debug, Clone)]
    struct SentenceWordQuery {
        // word info, some sentence words don't have database words associated with them
        word_id: Option<i32> = words::id.nullable(),
        word: Option<String> = words::word.nullable(),
        reading: Option<String> = words::reading.nullable(),
        // postgres doesn't support non-null constraints on array elements,
        // so these are Options even though they're never None
        furigana: Option<Vec<Option<database::Furigana>>> = words::furigana.nullable(),
        // postgres doesn't support non-null constraints on array elements,
        // so these are Options even though they're never None
        translations: Option<Vec<Option<String>>> = words::translations.nullable(),

        // sentence info
        sentence_id: i32 = sentences::id,
        sentence: String = sentences::sentence,
        sentence_word_reading: Option<String> = sentence_words::reading,
        // postgres doesn't support non-null constraints on array elements,
        // so these are Options even though they're never None
        sentence_word_furigana: Vec<Option<database::Furigana>> = sentence_words::furigana,
        idx_start: i32 = sentence_words::idx_start,
        idx_end: i32 = sentence_words::idx_end,
    }
}

crate::query! {
    #[derive(Debug, Clone)]
    struct KanjiWordQuery {
        kanji_id: i32 = kanji::id,
        kanji: String = kanji::chara,
        kanji_name: Option<String> = kanji::name,
        written_form: String = words::word,
        // postgres doesn't support non-null constraints on array elements,
        // so these are Options even though they're never None
        translations: Vec<Option<String>> = words::translations,
    }
}

crate::query! {
    #[derive(Debug, Clone)]
    struct SimilarKanjiQuery {
        lower_kanji_id: i32 = kanji_similar::lower_kanji_id,
        higher_kanji_id: i32 = kanji_similar::higher_kanji_id,
        kanji: String = kanji::chara,
        name: Option<String> = kanji::name,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::database::Furigana as DbFurigana;
    use lbr::anki::Furigana as AnkiFurigana;

    #[test]
    fn makes_word_card_from_query() {
        let word_id = Some(1);
        let sentence_id = 2;
        let query = SentenceWordQuery {
            word_id,
            word: Some("猫".to_string()),
            sentence: "吾輩は猫である".to_string(),
            sentence_word_reading: Some("ねこ".to_string()),
            sentence_word_furigana: vec![],
            idx_start: 9,
            idx_end: 12,
            reading: Some("ねこ".to_string()),
            furigana: Some(vec![Some(DbFurigana {
                word_start_idx: 0,
                word_end_idx: 3,
                reading_start_idx: 0,
                reading_end_idx: 6,
            })]),
            translations: Some(vec![Some("Cat".to_string())]),
            sentence_id,
        };
        let qs = vec![
            SentenceWordQuery {
                word_id,
                word: Some("吾輩".to_string()),
                sentence: "吾輩は猫である".to_string(),
                sentence_word_reading: Some("わがはい".to_string()),
                sentence_word_furigana: vec![],
                idx_start: 0,
                idx_end: 6,
                reading: Some("わがはい".to_string()),
                furigana: Some(vec![
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
                ]),
                translations: Some(vec![Some("I".to_string())]),
                sentence_id,
            },
            SentenceWordQuery {
                word_id,
                word: Some("は".to_string()),
                sentence: "吾輩は猫である".to_string(),
                sentence_word_reading: None,
                sentence_word_furigana: vec![],
                idx_start: 6,
                idx_end: 9,
                reading: Some("は".to_string()),
                furigana: Some(vec![]),
                translations: Some(vec![Some("tldr".to_string())]),
                sentence_id,
            },
            SentenceWordQuery {
                word_id,
                word: Some("猫".to_string()),
                sentence: "吾輩は猫である".to_string(),
                sentence_word_reading: Some("ねこ".to_string()),
                sentence_word_furigana: vec![],
                idx_start: 9,
                idx_end: 12,
                reading: Some("ねこ".to_string()),
                furigana: Some(vec![Some(DbFurigana {
                    word_start_idx: 0,
                    word_end_idx: 3,
                    reading_start_idx: 0,
                    reading_end_idx: 6,
                })]),
                translations: Some(vec![Some("Cat".to_string())]),
                sentence_id,
            },
            SentenceWordQuery {
                word_id,
                word: Some("で".to_string()),
                sentence: "吾輩は猫である".to_string(),
                sentence_word_reading: None,
                sentence_word_furigana: vec![],
                idx_start: 12,
                idx_end: 15,
                reading: Some("で".to_string()),
                furigana: Some(vec![]),
                translations: Some(vec![Some("something".to_string())]),
                sentence_id,
            },
            SentenceWordQuery {
                word_id,
                word: Some("ある".to_string()),
                sentence: "吾輩は猫である".to_string(),
                sentence_word_reading: None,
                sentence_word_furigana: vec![],
                idx_start: 15,
                idx_end: 18,
                reading: Some("ある".to_string()),
                furigana: Some(vec![]),
                translations: Some(vec![]),
                sentence_id,
            },
        ];

        let mut kanji_names_by_kanji = HashMap::new();
        kanji_names_by_kanji.insert("猫".to_string(), Some("cat".to_string()));

        let qs = qs.iter().collect::<Vec<_>>();
        let card = word_card_from_query(&query, &qs, &kanji_names_by_kanji, 1);
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
