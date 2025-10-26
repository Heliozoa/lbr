#!/bin/nu
use common.nu *
use backup.nu
use prepare-lbr-db.nu [
    prepare_lbr_db
]

export def restore_lbr_dump_path_prompt []: nothing -> string {
    return (input "lbr dump path: ")
}

export def restore_connection_user_prompt []: nothing -> string {
    return (input_default "lbr database user" "lbr")
}

export def main [] {
    let lbr_dump = restore_lbr_dump_path_prompt
    let lbr_user = restore_connection_user_prompt

    print "Backing up databases"
    backup

    print "Dropping lbr"
    print "Asking for postgres password"
    dropdb --username=postgres --if-exists lbr

    print "Initializing lbr"
    let lbr_db = "postgres://lbr:lbr@localhost/lbr"
    prepare_lbr_db $lbr_db
    psql $lbr_db -f $lbr_dump
}
