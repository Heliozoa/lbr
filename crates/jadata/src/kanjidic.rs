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
    #[serde(rename = "@cp_type")]
    pub cp_type: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Radical {
    pub rad_value: Vec<RadValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RadValue {
    #[serde(rename = "@rad_type")]
    pub rad_type: String,
    #[serde(rename = "#text")]
    pub text: u8,
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
    #[serde(rename = "@var_type")]
    pub var_type: String,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DicNumber {
    pub dic_ref: Vec<DicRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DicRef {
    #[serde(rename = "@dr_type")]
    pub dr_type: String,
    #[serde(rename = "@m_vol")]
    pub m_vol: Option<String>,
    #[serde(rename = "@m_page")]
    pub m_page: Option<String>,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryCode {
    pub q_code: Vec<QCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QCode {
    #[serde(rename = "@qc_type")]
    pub qc_type: String,
    #[serde(rename = "@skip_misclass")]
    pub skip_misclass: Option<String>,
    #[serde(rename = "#text")]
    pub text: String,
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
    #[serde(rename = "@r_type")]
    pub r_type: String,
    #[serde(rename = "@on_type")]
    pub on_type: Option<String>,
    #[serde(rename = "@r_status")]
    pub r_status: Option<String>,
    #[serde(rename = "#text")]
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meaning {
    #[serde(rename = "@m_lang")]
    pub m_lang: Option<String>,
    #[serde(rename = "#text")]
    pub text: String,
}
