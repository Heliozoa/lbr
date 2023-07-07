use crate::kanjifile::Kanji;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KanjifileManual {
    pub header: Header,
    pub kanji: Vec<Kanji>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub version: String,
}
