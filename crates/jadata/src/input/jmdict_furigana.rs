//! Models and parses the JmdictFurigana file.
//! See <https://github.com/Doublevil/JmdictFurigana>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Furigana {
    pub text: String,
    pub reading: String,
    pub furigana: Vec<Ruby>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ruby {
    pub ruby: String,
    pub rt: Option<String>,
}
