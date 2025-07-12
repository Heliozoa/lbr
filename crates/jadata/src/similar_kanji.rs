//! Models and parses the similar-kanji file.
//! See <https://github.com/siikamiika/similar-kanji>

use std::{collections::HashMap, path::Path};

pub fn parse(path: &Path) -> eyre::Result<HashMap<String, Vec<String>>> {
    let mut map = HashMap::new();

    let data = std::fs::read_to_string(path)?;
    for line in data.lines() {
        let mut split = line.split("/");
        let Some(first) = split.next() else {
            continue;
        };
        let similar = split
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        map.insert(first.to_string(), similar);
    }

    Ok(map)
}
