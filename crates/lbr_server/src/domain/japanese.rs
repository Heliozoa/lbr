//! Functionality related to the Japanese language.

use crate::{
    query,
    schema::{kanji as k, kanji_readings as kr},
    utils::database::Furigana,
};
use diesel::prelude::*;
use std::collections::HashMap;

/// Returns a mapping from kanji to its potential readings.
pub fn kanji_to_readings(conn: &mut PgConnection) -> eyre::Result<HashMap<String, Vec<String>>> {
    tracing::info!("Building a mapping from kanji to its readings");

    let kanji_with_reading = k::table
        .inner_join(kr::table.on(kr::kanji_id.eq(k::id)))
        .select(KanjiWithReading::as_select())
        .get_results(conn)?;
    let mut kanji_to_readings: HashMap<String, Vec<String>> = HashMap::new();
    for kwr in kanji_with_reading {
        kanji_to_readings
            .entry(kwr.kanji)
            .or_default()
            .push(kwr.reading);
    }
    Ok(kanji_to_readings)
}

query! {
    struct KanjiWithReading {
        kanji: String = kanji::chara,
        reading: String = kanji_readings::reading,
    }
}

/// Maps a reading onto a word with the database `Furigana` type.
pub fn map_to_db_furigana(
    word: &str,
    reading: &str,
    kanji_to_readings: &HashMap<String, Vec<String>>,
) -> eyre::Result<Vec<Furigana>> {
    let furigana = furigana::map(word, reading, kanji_to_readings);
    let furigana = if furigana.is_empty() {
        tracing::warn!("Failed to map furigana accurately for '{word}' with reading '{reading}', using naive mapping");
        furigana::map_naive(word, reading)
    } else {
        furigana
    };
    let furigana = furigana
        .into_iter()
        .max_by_key(|f| f.accuracy)
        .map(furigana_to_db_furigana)
        .ok_or_else(|| eyre::eyre!("Failed to assign furigana to {word} with {reading}"))??;
    Ok(furigana)
}

// converts furigana to the database form
fn furigana_to_db_furigana(furigana: furigana::Furigana) -> eyre::Result<Vec<Furigana>> {
    let mut f = Vec::new();

    let mut word_idx = 0;
    let mut reading_idx = 0;
    for segment in furigana.furigana {
        let segment_len = i32::try_from(segment.segment.len())?;
        let word_end_idx = word_idx + segment_len;
        if let Some(segment_furigana) = segment.furigana {
            let reading_end_idx = reading_idx + i32::try_from(segment_furigana.len())?;
            f.push(Furigana {
                word_start_idx: word_idx,
                word_end_idx,
                reading_start_idx: reading_idx,
                reading_end_idx,
            });
            reading_idx = reading_end_idx;
        } else {
            reading_idx += segment_len;
        }
        word_idx = word_end_idx;
    }
    Ok(f)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn converts_furigana_to_db_furigana() {
        use crate::utils::database::Furigana as DbFurigana;
        use furigana::{Furigana, FuriganaSegment};

        let furigana = vec![
            FuriganaSegment {
                segment: "物",
                furigana: Some("もの"),
            },
            FuriganaSegment {
                segment: "の",
                furigana: None,
            },
            FuriganaSegment {
                segment: "怪",
                furigana: Some("け"),
            },
        ];
        let furigana = Furigana {
            furigana,
            accuracy: 1,
        };
        let furigana = furigana_to_db_furigana(furigana).unwrap();
        assert_eq!(furigana.len(), 2, "{furigana:#?}");
        assert_eq!(
            furigana[0],
            DbFurigana {
                word_start_idx: 0,
                word_end_idx: 3,
                reading_start_idx: 0,
                reading_end_idx: 6
            },
            "{:#?}",
            furigana[0]
        );
        assert_eq!(
            furigana[1],
            DbFurigana {
                word_start_idx: 6,
                word_end_idx: 9,
                reading_start_idx: 9,
                reading_end_idx: 12
            },
            "{:#?}",
            furigana[1]
        );

        assert_eq!(
            &"物の怪"[furigana[0].word_start_idx as usize..furigana[0].word_end_idx as usize],
            "物"
        );
        assert_eq!(
            &"もののけ"
                [furigana[0].reading_start_idx as usize..furigana[0].reading_end_idx as usize],
            "もの"
        );
        assert_eq!(
            &"物の怪"[furigana[1].word_start_idx as usize..furigana[1].word_end_idx as usize],
            "怪"
        );
        assert_eq!(
            &"もののけ"
                [furigana[1].reading_start_idx as usize..furigana[1].reading_end_idx as usize],
            "け"
        );
    }

    #[test]
    fn converts_furigana_to_db_furigana2() {
        use crate::utils::database::Furigana as DbFurigana;
        use furigana::{Furigana, FuriganaSegment};

        let furigana = vec![
            FuriganaSegment {
                segment: "近",
                furigana: Some("きん"),
            },
            FuriganaSegment {
                segment: "所",
                furigana: Some("じょ"),
            },
        ];
        let furigana = Furigana {
            furigana,
            accuracy: 1,
        };
        let furigana = furigana_to_db_furigana(furigana).unwrap();
        assert_eq!(furigana.len(), 2, "{furigana:#?}");
        assert_eq!(
            furigana[0],
            DbFurigana {
                word_start_idx: 0,
                word_end_idx: 3,
                reading_start_idx: 0,
                reading_end_idx: 6
            },
            "{:#?}",
            furigana[0]
        );
        assert_eq!(
            furigana[1],
            DbFurigana {
                word_start_idx: 3,
                word_end_idx: 6,
                reading_start_idx: 6,
                reading_end_idx: 12
            },
            "{:#?}",
            furigana[1]
        );

        assert_eq!(
            &"近所"[furigana[0].word_start_idx as usize..furigana[0].word_end_idx as usize],
            "近"
        );
        assert_eq!(
            &"きんじょ"
                [furigana[0].reading_start_idx as usize..furigana[0].reading_end_idx as usize],
            "きん"
        );
        assert_eq!(
            &"近所"[furigana[1].word_start_idx as usize..furigana[1].word_end_idx as usize],
            "所"
        );
        assert_eq!(
            &"きんじょ"
                [furigana[1].reading_start_idx as usize..furigana[1].reading_end_idx as usize],
            "じょ"
        );
    }
}
