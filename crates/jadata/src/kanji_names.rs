use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct KanjiNames {
    pub kanji_names: HashMap<String, String>,
}
