#!/bin/nu
use common.nu *

use prepare-ichiran-cli.nu [
    prepare_ichiran_cli
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
use prepare-ichiran-seq-to-word-id.nu

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
        let ichiran_database = $"\(\"($env.ICHIRAN_CONNECTION_NAME)\"\)"
        prepare_ichiran_db $env.ICHIRAN_CONNECTION_NAME $env.ICHIRAN_CONNECTION_USER "./data/ichiran.pgdump"
        prepare_ichiran_cli $env.ICHIRAN_CONNECTION_NAME $env.ICHIRAN_CONNECTION_USER $env.ICHIRAN_CONNECTION_PASSWORD $env.ICHIRAN_CONNECTION_HOST "./data/jmdictdb"
        prepare-ichiran-seq-to-word-id

        print "Updating lbr data"
        timeit {
            (cargo run --release --bin update_db --
                "./data/kanjidic2.xml"
                "./data/kradfile"
                "./crates/jadata/data/kanji_names.json"
                "./crates/jadata/data/kanji_similar.json"
                "./crates/jadata/data/kanji_extra.json"
                "./data/JMdict_e_examp.xml")
            | complete
            | check_error
        }
    }
}
