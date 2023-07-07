//! Models and parses the JMdict file.
//! See <https://www.edrdg.org/wiki/index.php/JMdict-EDICT_Dictionary_Project>

use serde::{Deserialize, Serialize};
use serde_xml_rs::{Deserializer, EventReader, ParserConfig};
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JMDict {
    #[serde(default)]
    pub entry: Vec<Entry>,
}

impl JMDict {
    pub fn deserialize<R>(r: R) -> Result<Self, serde_xml_rs::Error>
    where
        R: Read,
    {
        let config = make_config();
        let reader = EventReader::new_with_config(r, config);
        <Self as Deserialize>::deserialize(&mut Deserializer::new(reader))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub ent_seq: String,
    #[serde(default)]
    pub k_ele: Vec<KEle>,
    pub r_ele: Vec<REle>,
    pub sense: Vec<Sense>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KEle {
    pub keb: String,
    #[serde(default)]
    pub ke_inf: Vec<String>,
    #[serde(default)]
    pub ke_pri: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct REle {
    pub reb: String,
    pub re_nokanji: Option<String>,
    #[serde(default)]
    pub re_restr: Vec<String>,
    #[serde(default)]
    pub re_inf: Vec<String>,
    #[serde(default)]
    pub re_pri: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sense {
    #[serde(default)]
    pub stagk: Vec<String>,
    #[serde(default)]
    pub stagr: Vec<String>,
    #[serde(default)]
    pub pos: Vec<String>,
    #[serde(default)]
    pub xref: Vec<String>,
    #[serde(default)]
    pub ant: Vec<String>,
    #[serde(default)]
    pub field: Vec<String>,
    #[serde(default)]
    pub misc: Vec<String>,
    #[serde(default)]
    pub s_inf: Vec<String>,
    #[serde(default)]
    pub lsource: Vec<Lsource>,
    #[serde(default)]
    pub dial: Vec<String>,
    #[serde(default)]
    pub gloss: Vec<Gloss>,
    #[serde(default)]
    pub example: Vec<Example>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lsource {
    #[serde(rename = "$value")]
    pub value: Option<String>,
    pub lang: Option<String>,
    pub ls_type: Option<String>,
    pub ls_wasei: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gloss {
    #[serde(rename = "$value")]
    pub value: String,
    pub lang: Option<String>,
    pub g_gend: Option<String>,
    pub g_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Example {
    pub ex_srce: ExSrce,
    pub ex_text: String,
    pub ex_sent: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExSrce {
    #[serde(rename = "$value")]
    pub value: String,
    pub exsrc_type: String,
}

fn make_config() -> ParserConfig {
    ParserConfig::new()
        .add_entity("bra", "Brazilian")
        .add_entity("hob", "Hokkaido-ben")
        .add_entity("ksb", "Kansai-ben")
        .add_entity("ktb", "Kantou-ben")
        .add_entity("kyb", "Kyoto-ben")
        .add_entity("kyu", "Kyuushuu-ben")
        .add_entity("nab", "Nagano-ben")
        .add_entity("osb", "Osaka-ben")
        .add_entity("rkb", "Ryuukyuu-ben")
        .add_entity("thb", "Touhoku-ben")
        .add_entity("tsb", "Tosa-ben")
        .add_entity("tsug", "Tsugaru-ben")
        // <field> entities
        .add_entity("agric", "agriculture")
        .add_entity("anat", "anatomy")
        .add_entity("archeol", "archeology")
        .add_entity("archit", "architecture")
        .add_entity("art", "art, aesthetics")
        .add_entity("astron", "astronomy")
        .add_entity("audvid", "audiovisual")
        .add_entity("aviat", "aviation")
        .add_entity("baseb", "baseball")
        .add_entity("biochem", "biochemistry")
        .add_entity("biol", "biology")
        .add_entity("bot", "botany")
        .add_entity("Buddh", "Buddhism")
        .add_entity("bus", "business")
        .add_entity("cards", "card games")
        .add_entity("chem", "chemistry")
        .add_entity("Christn", "Christianity")
        .add_entity("cloth", "clothing")
        .add_entity("comp", "computing")
        .add_entity("cryst", "crystallography")
        .add_entity("dent", "dentistry")
        .add_entity("ecol", "ecology")
        .add_entity("econ", "economics")
        .add_entity("elec", "electricity, elec. eng.")
        .add_entity("electr", "electronics")
        .add_entity("embryo", "embryology")
        .add_entity("engr", "engineering")
        .add_entity("ent", "entomology")
        .add_entity("film", "film")
        .add_entity("finc", "finance")
        .add_entity("fish", "fishing")
        .add_entity("food", "food, cooking")
        .add_entity("gardn", "gardening, horticulture")
        .add_entity("genet", "genetics")
        .add_entity("geogr", "geography")
        .add_entity("geol", "geology")
        .add_entity("geom", "geometry")
        .add_entity("go", "go (game)")
        .add_entity("golf", "golf")
        .add_entity("gramm", "grammar")
        .add_entity("grmyth", "Greek mythology")
        .add_entity("hanaf", "hanafuda")
        .add_entity("horse", "horse racing")
        .add_entity("kabuki", "kabuki")
        .add_entity("law", "law")
        .add_entity("ling", "linguistics")
        .add_entity("logic", "logic")
        .add_entity("MA", "martial arts")
        .add_entity("mahj", "mahjong")
        .add_entity("manga", "manga")
        .add_entity("math", "mathematics")
        .add_entity("mech", "mechanical engineering")
        .add_entity("med", "medicine")
        .add_entity("met", "meteorology")
        .add_entity("mil", "military")
        .add_entity("mining", "mining")
        .add_entity("music", "music")
        .add_entity("noh", "noh")
        .add_entity("ornith", "ornithology")
        .add_entity("paleo", "paleontology")
        .add_entity("pathol", "pathology")
        .add_entity("pharm", "pharmacology")
        .add_entity("phil", "philosophy")
        .add_entity("photo", "photography")
        .add_entity("physics", "physics")
        .add_entity("physiol", "physiology")
        .add_entity("politics", "politics")
        .add_entity("print", "printing")
        .add_entity("psy", "psychiatry")
        .add_entity("psyanal", "psychoanalysis")
        .add_entity("psych", "psychology")
        .add_entity("rail", "railway")
        .add_entity("rommyth", "Roman mythology")
        .add_entity("Shinto", "Shinto")
        .add_entity("shogi", "shogi")
        .add_entity("ski", "skiing")
        .add_entity("sports", "sports")
        .add_entity("stat", "statistics")
        .add_entity("stockm", "stock market")
        .add_entity("sumo", "sumo")
        .add_entity("telec", "telecommunications")
        .add_entity("tradem", "trademark")
        .add_entity("tv", "television")
        .add_entity("vidg", "video games")
        .add_entity("zool", "zoology")
        // <ke_inf> (kanji info) entities
        .add_entity("ateji", "ateji (phonetic) reading")
        .add_entity("ik", "word containing irregular kana usage")
        .add_entity("iK", "word containing irregular kanji usage")
        .add_entity("io", "irregular okurigana usage")
        .add_entity("oK", "word containing out-dated kanji or kanji usage")
        .add_entity("rK", "rarely-used kanji form")
        .add_entity("sK", "search-only kanji form")
        // <misc> (miscellaneous) entities
        .add_entity("abbr", "abbreviation")
        .add_entity("arch", "archaic")
        .add_entity("char", "character")
        .add_entity("chn", "children's language")
        .add_entity("col", "colloquial")
        .add_entity("company", "company name")
        .add_entity("creat", "creature")
        .add_entity("dated", "dated term")
        .add_entity("dei", "deity")
        .add_entity("derog", "derogatory")
        .add_entity("doc", "document")
        .add_entity("euph", "euphemistic")
        .add_entity("ev", "event")
        .add_entity("fam", "familiar language")
        .add_entity("fem", "female term or language")
        .add_entity("fict", "fiction")
        .add_entity("form", "formal or literary term")
        .add_entity("given", "given name or forename, gender not specified")
        .add_entity("group", "group")
        .add_entity("hist", "historical term")
        .add_entity("hon", "honorific or respectful (sonkeigo) language")
        .add_entity("hum", "humble (kenjougo) language")
        .add_entity("id", "idiomatic expression")
        .add_entity("joc", "jocular, humorous term")
        .add_entity("leg", "legend")
        .add_entity("m-sl", "manga slang")
        .add_entity("male", "male term or language")
        .add_entity("myth", "mythology")
        .add_entity("net-sl", "Internet slang")
        .add_entity("obj", "object")
        .add_entity("obs", "obsolete term")
        .add_entity("on-mim", "onomatopoeic or mimetic word")
        .add_entity("organization", "organization name")
        .add_entity("oth", "other")
        .add_entity("person", "full name of a particular person")
        .add_entity("place", "place name")
        .add_entity("poet", "poetical term")
        .add_entity("pol", "polite (teineigo) language")
        .add_entity("product", "product name")
        .add_entity("proverb", "proverb")
        .add_entity("quote", "quotation")
        .add_entity("rare", "rare term")
        .add_entity("relig", "religion")
        .add_entity("sens", "sensitive")
        .add_entity("serv", "service")
        .add_entity("ship", "ship name")
        .add_entity("sl", "slang")
        .add_entity("station", "railway station")
        .add_entity("surname", "family or surname")
        .add_entity("uk", "word usually written using kana alone")
        .add_entity("unclass", "unclassified name")
        .add_entity("vulg", "vulgar expression or word")
        .add_entity("work", "work of art, literature, music, etc. name")
        .add_entity(
            "X",
            "rude or X-rated term (not displayed in educational software)",
        )
        .add_entity("yoji", "yojijukugo")
        // <pos> (part-of-speech) entities
        .add_entity("adj-f", "noun or verb acting prenominally")
        .add_entity("adj-i", "adjective (keiyoushi)")
        .add_entity("adj-ix", "adjective (keiyoushi) - yoi/ii class")
        .add_entity("adj-kari", "'kari' adjective (archaic)")
        .add_entity("adj-ku", "'ku' adjective (archaic)")
        .add_entity(
            "adj-na",
            "adjectival nouns or quasi-adjectives (keiyodoshi)",
        )
        .add_entity("adj-nari", "archaic/formal form of na-adjective")
        .add_entity(
            "adj-no",
            "nouns which may take the genitive case particle 'no'",
        )
        .add_entity("adj-pn", "pre-noun adjectival (rentaishi)")
        .add_entity("adj-shiku", "'shiku' adjective (archaic)")
        .add_entity("adj-t", "'taru' adjective")
        .add_entity("adv", "adverb (fukushi)")
        .add_entity("adv-to", "adverb taking the 'to' particle")
        .add_entity("aux", "auxiliary")
        .add_entity("aux-adj", "auxiliary adjective")
        .add_entity("aux-v", "auxiliary verb")
        .add_entity("conj", "conjunction")
        .add_entity("cop", "copula")
        .add_entity("ctr", "counter")
        .add_entity("exp", "expressions (phrases, clauses, etc.)")
        .add_entity("int", "interjection (kandoushi)")
        .add_entity("n", "noun (common) (futsuumeishi)")
        .add_entity("n-adv", "adverbial noun (fukushitekimeishi)")
        .add_entity("n-pr", "proper noun")
        .add_entity("n-pref", "noun, used as a prefix")
        .add_entity("n-suf", "noun, used as a suffix")
        .add_entity("n-t", "noun (temporal) (jisoumeishi)")
        .add_entity("num", "numeric")
        .add_entity("pn", "pronoun")
        .add_entity("pref", "prefix")
        .add_entity("prt", "particle")
        .add_entity("suf", "suffix")
        .add_entity("unc", "unclassified")
        .add_entity("v-unspec", "verb unspecified")
        .add_entity("v1", "Ichidan verb")
        .add_entity("v1-s", "Ichidan verb - kureru special class")
        .add_entity("v2a-s", "Nidan verb with 'u' ending (archaic)")
        .add_entity(
            "v2b-k",
            "Nidan verb (upper class) with 'bu' ending (archaic)",
        )
        .add_entity(
            "v2b-s",
            "Nidan verb (lower class) with 'bu' ending (archaic)",
        )
        .add_entity(
            "v2d-k",
            "Nidan verb (upper class) with 'dzu' ending (archaic)",
        )
        .add_entity(
            "v2d-s",
            "Nidan verb (lower class) with 'dzu' ending (archaic)",
        )
        .add_entity(
            "v2g-k",
            "Nidan verb (upper class) with 'gu' ending (archaic)",
        )
        .add_entity(
            "v2g-s",
            "Nidan verb (lower class) with 'gu' ending (archaic)",
        )
        .add_entity(
            "v2h-k",
            "Nidan verb (upper class) with 'hu/fu' ending (archaic)",
        )
        .add_entity(
            "v2h-s",
            "Nidan verb (lower class) with 'hu/fu' ending (archaic)",
        )
        .add_entity(
            "v2k-k",
            "Nidan verb (upper class) with 'ku' ending (archaic)",
        )
        .add_entity(
            "v2k-s",
            "Nidan verb (lower class) with 'ku' ending (archaic)",
        )
        .add_entity(
            "v2m-k",
            "Nidan verb (upper class) with 'mu' ending (archaic)",
        )
        .add_entity(
            "v2m-s",
            "Nidan verb (lower class) with 'mu' ending (archaic)",
        )
        .add_entity(
            "v2n-s",
            "Nidan verb (lower class) with 'nu' ending (archaic)",
        )
        .add_entity(
            "v2r-k",
            "Nidan verb (upper class) with 'ru' ending (archaic)",
        )
        .add_entity(
            "v2r-s",
            "Nidan verb (lower class) with 'ru' ending (archaic)",
        )
        .add_entity(
            "v2s-s",
            "Nidan verb (lower class) with 'su' ending (archaic)",
        )
        .add_entity(
            "v2t-k",
            "Nidan verb (upper class) with 'tsu' ending (archaic)",
        )
        .add_entity(
            "v2t-s",
            "Nidan verb (lower class) with 'tsu' ending (archaic)",
        )
        .add_entity(
            "v2w-s",
            "Nidan verb (lower class) with 'u' ending and 'we' conjugation (archaic)",
        )
        .add_entity(
            "v2y-k",
            "Nidan verb (upper class) with 'yu' ending (archaic)",
        )
        .add_entity(
            "v2y-s",
            "Nidan verb (lower class) with 'yu' ending (archaic)",
        )
        .add_entity(
            "v2z-s",
            "Nidan verb (lower class) with 'zu' ending (archaic)",
        )
        .add_entity("v4b", "Yodan verb with 'bu' ending (archaic)")
        .add_entity("v4g", "Yodan verb with 'gu' ending (archaic)")
        .add_entity("v4h", "Yodan verb with 'hu/fu' ending (archaic)")
        .add_entity("v4k", "Yodan verb with 'ku' ending (archaic)")
        .add_entity("v4m", "Yodan verb with 'mu' ending (archaic)")
        .add_entity("v4n", "Yodan verb with 'nu' ending (archaic)")
        .add_entity("v4r", "Yodan verb with 'ru' ending (archaic)")
        .add_entity("v4s", "Yodan verb with 'su' ending (archaic)")
        .add_entity("v4t", "Yodan verb with 'tsu' ending (archaic)")
        .add_entity("v5aru", "Godan verb - -aru special class")
        .add_entity("v5b", "Godan verb with 'bu' ending")
        .add_entity("v5g", "Godan verb with 'gu' ending")
        .add_entity("v5k", "Godan verb with 'ku' ending")
        .add_entity("v5k-s", "Godan verb - Iku/Yuku special class")
        .add_entity("v5m", "Godan verb with 'mu' ending")
        .add_entity("v5n", "Godan verb with 'nu' ending")
        .add_entity("v5r", "Godan verb with 'ru' ending")
        .add_entity("v5r-i", "Godan verb with 'ru' ending (irregular verb)")
        .add_entity("v5s", "Godan verb with 'su' ending")
        .add_entity("v5t", "Godan verb with 'tsu' ending")
        .add_entity("v5u", "Godan verb with 'u' ending")
        .add_entity("v5u-s", "Godan verb with 'u' ending (special class)")
        .add_entity("v5uru", "Godan verb - Uru old class verb (old form of Eru)")
        .add_entity("vi", "intransitive verb")
        .add_entity("vk", "Kuru verb - special class")
        .add_entity("vn", "irregular nu verb")
        .add_entity("vr", "irregular ru verb, plain form ends with -ri")
        .add_entity("vs", "noun or participle which takes the aux. verb suru")
        .add_entity("vs-c", "su verb - precursor to the modern suru")
        .add_entity("vs-i", "suru verb - included")
        .add_entity("vs-s", "suru verb - special class")
        .add_entity("vt", "transitive verb")
        .add_entity(
            "vz",
            "Ichidan verb - zuru verb (alternative form of -jiru verbs)",
        )
        // <re_inf> (reading info) entities
        .add_entity(
            "gikun",
            "gikun (meaning as reading) or jukujikun (special kanji reading)",
        )
        .add_entity("ik", "word containing irregular kana usage")
        .add_entity("ok", "out-dated or obsolete kana usage")
        .add_entity("sk", "search-only kana form")
}
