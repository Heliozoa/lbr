use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct KanjiSimilar {
    pub kanji_similar: HashMap<String, Vec<String>>,
}
