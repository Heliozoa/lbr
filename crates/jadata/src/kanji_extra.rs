use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KanjiExtra {
    pub kanji_extra: Vec<ExtraKanji>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtraKanji {
    pub chara: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub meanings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub readings: Vec<String>,
}
