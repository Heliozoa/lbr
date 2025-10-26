#!/bin/nu
use common.nu *

# Initialises the ichiran database.
def main [] {
    let database_name = prepare_ichiran_db_name_prompt
    let database_user = prepare_ichiran_db_user_prompt
    let database_dump = prepare_ichiran_db_dump_prompt
    prepare_ichiran_db $database_name $database_user $database_dump
}

export def prepare_ichiran_db_name_prompt []: nothing -> string {
    return (input_default "ichiran database name" "ichiran")
}

export def prepare_ichiran_db_user_prompt []: nothing -> string {
    return (input_default "ichiran database user" "lbr")
}

export def prepare_ichiran_db_dump_prompt []: nothing -> string {
    return (input_default "ichiran database dump path" "./data/ichiran.pgdump")
}

# Initialises the ichiran database.
export def prepare_ichiran_db [database_name: string, database_user: string, database_dump: string]: nothing -> string {
    print $"Dropping database `($database_name)`"
    print "Asking password for postgres (dropdb)"
    dropdb --username=postgres --if-exists $database_name
        | complete
        | check_error

    print "Creating database"
    print "Asking password for postgres (createdb)"
    createdb --username=postgres --owner=($database_user) --encoding='UTF8' --locale='ja_JP.utf8' --template=template0 $database_name
        | complete
        | check_error

    print "Restoring database, this may take a while"
    print "Asking password for postgres (pg_restore)"
    pg_restore --clean --if-exists --no-owner --role=($database_user) --username=postgres --dbname=($database_name) $database_dump
        | complete
        # pg_restore will probably report errors that we don't care about, so we'll only warn here
        | check_warning
}
