//! Functionality related to the Japanese language.

use crate::{
    error::EyreResult,
    query,
    schema::{kanji as k, kanji_readings as kr},
    utils::database::Furigana,
    LbrPool,
};
use diesel::prelude::*;
use std::collections::HashMap;

query! {
    struct KanjiWithReading {
        kanji: String = kanji::chara,
        reading: String = kanji_readings::reading,
    }
}

/// Returns a mapping from kanji to its potential readings.
pub async fn kanji_to_readings(pool: LbrPool) -> eyre::Result<HashMap<String, Vec<String>>> {
    let ktr = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get()?;
        let kanji_with_reading = k::table
            .inner_join(kr::table.on(kr::kanji_id.eq(k::id)))
            .select(KanjiWithReading::as_select())
            .get_results(&mut conn)?;
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for kwr in kanji_with_reading {
            map.entry(kwr.kanji).or_default().push(kwr.reading);
        }
        EyreResult::Ok(map)
    })
    .await??;
    Ok(ktr)
}

/// Maps a reading onto a word with the database `Furigana` type.
pub fn map_to_db_furigana(
    word: &str,
    reading: &str,
    kanji_to_readings: &HashMap<String, Vec<String>>,
) -> eyre::Result<Vec<Furigana>> {
    let furigana = furigana::map(word, reading, kanji_to_readings)
        .into_iter()
        .max_by_key(|f| f.accuracy)
        .map(|f| furigana_to_db_furigana(f))
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
}
