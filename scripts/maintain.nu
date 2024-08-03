use common.nu *
use prepare-ichiran.nu [
    prepare_ichiran
]
use downloads.nu [
    dl_ichiran_dump
    dl_jmdict
    dl_kanjidic
    dl_kradfile
]
use prepare-ichiran-db.nu [
    prepare_ichiran_db
]

# Updates the data sources used by the project without throwing out existing lbr data.
export def main [] {
    initialise_logging
    with-env (env-vars) {
        print "Updating data files"
        dl_ichiran_dump "./data/ichiran.pgdump"
        dl_jmdict "./data/JMdict_e_examp.xml"
        dl_kanjidic "./data/kanjidic2.xml"
        dl_kradfile "./data/kradfile"

        print "Updating ichiran data"
        prepare_ichiran $env.ICHIRAN_DATABASE_URL "./data/jmdictdb"
        prepare_ichiran_db $env.ICHIRAN_DATABASE_NAME $env.ICHIRAN_DATABASE_USER "./data/ichiran.pgdump"
        prepare-ichiran-seq-to-word-id

        print "Updating lbr data"
        dl_jmdict "./data/JMdict_e_examp.xml"
        dl_kanjidic "./data/kanjidic2.xml"
        dl_kradfile "./data/kradfile"
        dl_jmdict_furigana "./data/JmdictFurigana.json"
        (timeit
            cargo run --release --bin update_db --
                "./data/kanjidic2.xml"
                "./data/kradfile"
                ./crates/jadata/data/kanji_names.json
                ./crates/jadata/data/kanji_similar.json
                ./crates/jadata/data/kanji_extra.json
                "./data/JMdict_e_examp.xml"
                "./data/JmdictFurigana.json"
        )
        prepare-ichiran-seq-to-word-id
    }
}