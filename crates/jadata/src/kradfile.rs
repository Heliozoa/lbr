//! Models and parses the KRADFILE.
//! See <https://www.edrdg.org/krad/kradinf.html>

use encoding_rs::EUC_JP;
use std::{collections::HashMap, io::Read};

pub struct Kradfile {
    pub kanji_to_components: HashMap<String, Vec<String>>,
}

impl Kradfile {
    pub fn from<R: Read>(mut r: R) -> eyre::Result<Self> {
        let mut buf = vec![];
        r.read_to_end(&mut buf)?;

        // the KRADFILE is EUC-JP encoded
        let text = EUC_JP
            .decode_without_bom_handling_and_without_replacement(&buf)
            .ok_or_else(|| eyre::eyre!("Invalid kradfile"))?;

        // the KRADFILE makes some concessions due to being EUC-JP encoded and uses placeholder characters for characters that exist in Unicode
        // here, we replace them with the proper Unicode characters
        let mut replacement_map = HashMap::new();
        replacement_map.insert("化", "\u{2E85}");
        replacement_map.insert("个", "\u{2F09}");
        replacement_map.insert("并", "upside-down ハ"); // this component does not have have a corresponding Unicode character
        replacement_map.insert("刈", "\u{2E89}");
        replacement_map.insert("込", "\u{2ECC}");
        replacement_map.insert("尚", "\u{2E8C}");
        replacement_map.insert("忙", "\u{2E96}");
        replacement_map.insert("扎", "\u{2E97}");
        replacement_map.insert("汁", "\u{2EA1}");
        replacement_map.insert("犯", "\u{2EA8}");
        replacement_map.insert("艾", "\u{2EBE}");
        replacement_map.insert("邦", "\u{2ECF}");
        replacement_map.insert("阡", "\u{2ED9}");
        replacement_map.insert("老", "\u{2EB9}");
        replacement_map.insert("杰", "\u{2EA3}");
        replacement_map.insert("礼", "\u{2EAD}");
        replacement_map.insert("疔", "\u{2F67}");
        replacement_map.insert("禹", "\u{2F71}");
        replacement_map.insert("初", "\u{2EC2}");
        replacement_map.insert("買", "\u{2EB2}");
        replacement_map.insert("滴", "\u{5547}");

        // the KRADFILE is formatted {kanji} : {component_1} {component_2} ...
        let mut kanji_to_components = HashMap::new();
        for line in text.lines() {
            if let Some((kanji, radicals)) = line.split_once(':') {
                let kanji = kanji.trim();
                let mut radicals = radicals
                    .split_whitespace()
                    .map(String::from)
                    .collect::<Vec<_>>();
                for radical in &mut radicals {
                    if let Some(val) = replacement_map.get(radical.as_str()) {
                        *radical = val.to_string();
                    }
                }
                kanji_to_components.insert(kanji.to_string(), radicals);
            }
        }

        Ok(Self {
            kanji_to_components,
        })
    }
}
