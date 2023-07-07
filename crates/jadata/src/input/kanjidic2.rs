//! Models and parses the KANJIDIC2 file.
//! See <https://www.edrdg.org/wiki/index.php/KANJIDIC_Project>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kanjidic2 {
    pub header: Header,
    #[serde(default)]
    pub character: Vec<Character>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Header {
    pub file_version: String,
    pub database_version: String,
    pub date_of_creation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Character {
    pub literal: String,
    pub codepoint: Codepoint,
    pub radical: Radical,
    pub misc: Misc,
    pub dic_number: Option<DicNumber>,
    pub query_code: Option<QueryCode>,
    pub reading_meaning: Option<ReadingMeaning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Codepoint {
    pub cp_value: Vec<CpValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CpValue {
    #[serde(rename = "$value")]
    pub value: String,
    pub cp_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Radical {
    pub rad_value: Vec<RadValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadValue {
    #[serde(rename = "$value")]
    pub value: u8,
    pub rad_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Misc {
    pub grade: Option<String>,
    pub stroke_count: Vec<String>,
    #[serde(default)]
    pub variant: Vec<Variant>,
    pub freq: Option<String>,
    #[serde(default)]
    pub rad_name: Vec<String>,
    pub jlpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Variant {
    #[serde(rename = "$value")]
    pub value: String,
    pub var_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DicNumber {
    pub dic_ref: Vec<DicRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DicRef {
    #[serde(rename = "$value")]
    pub value: String,
    pub dr_type: String,
    pub m_vol: Option<String>,
    pub m_page: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryCode {
    pub q_code: Vec<QCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QCode {
    #[serde(rename = "$value")]
    pub value: String,

    pub qc_type: String,
    pub skip_misclass: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingMeaning {
    #[serde(default)]
    pub rmgroup: Vec<Rmgroup>,
    #[serde(default)]
    pub nanori: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rmgroup {
    #[serde(default)]
    pub reading: Vec<Reading>,
    #[serde(default)]
    pub meaning: Vec<Meaning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reading {
    #[serde(rename = "$value")]
    pub value: String,
    pub r_type: String,
    pub on_type: Option<String>,
    pub r_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meaning {
    #[serde(rename = "$value")]
    pub value: String,
    pub m_lang: Option<String>,
}
