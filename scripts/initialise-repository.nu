#!/bin/nu
use common.nu *

use prepare-ichiran-cli.nu [
    prepare_ichiran_cli
    prepare_ichiran_cli_prompt
]
use prepare-ichiran-db.nu [
    prepare_ichiran_db_name_prompt
    prepare_ichiran_db_user_prompt
    prepare_ichiran_db_dump_prompt
    prepare_ichiran_db
]
use prepare-ichiran-seq-to-word-id.nu
use prepare-lbr-db-user.nu
use prepare-lbr-db.nu [
    prepare_lbr_db_prompt
    prepare_lbr_db
]
use generate-license.nu [
    generate_license
]
use downloads.nu

# Initialises everything the project needs to function from scratch.
def main [] {
    initialise_logging
    print "Initialising repository"

    let ichiran_dump_path = input_default "Ichiran dump download path" "./data/ichiran.pgdump"
    let jmdict_path = input_default "JMdict download path" "./data/JMdict_e_examp.xml"
    let jmdictdb_path = input_default "JMdict download path" "./data/jmdictdb"
    let kanjidic_path = input_default "KANJIDIC2 download path" "./data/kanjidic2.xml"
    let kradfile_path = input_default "KRADFILE download path" "./data/kradfile"
    let jmdict_furigana_path = input_default "JmdictFurigana download path" "./data/JmdictFurigana.json"

    dl_ichiran_dump $ichiran_dump_path
    dl_jmdict $jmdict_path
    dl_kanjidic $kanjidic_path
    dl_kradfile $kradfile_path
    dl_jmdict_furigana $jmdict_furigana_path

    let ichiran_connection = prepare_ichiran_cli_prompt
    let ichiran_database_name = prepare_ichiran_db_name_prompt
    let ichiran_database_user = prepare_ichiran_db_user_prompt
    let ichiran_database_password = input_password "ichiran database"
    let ichiran_database_dump = prepare_ichiran_db_dump_prompt
    let lbr_connection = prepare_lbr_db_prompt

    prepare_ichiran_cli $ichiran_connection $jmdictdb_path
    prepare_ichiran_db $ichiran_database_name $ichiran_database_user $ichiran_database_dump
    prepare-ichiran-seq-to-word-id

    prepare-lbr-db-user
    prepare_lbr_db $lbr_connection

    generate_license "web"
    generate_license "docker"

    (generate_env
        $lbr_connection
        $ichiran_connection
        $ichiran_database_name
        $ichiran_database_user
        $ichiran_database_password
    )
}
