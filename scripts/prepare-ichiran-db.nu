#!/bin/nu

use common.nu *

# Initialises the ichiran database.
def main [] {
    let database_name = prepare_ichiran_db_name_prompt
    let database_user = prepare_ichiran_db_user_prompt
    let database_dump = prepare_ichiran_db_dump_prompt
    prepare_ichiran_db $database_name $database_user $database_dump
}

export def prepare_ichiran_db_name_prompt [] -> string {
    return (input_default "ichiran database name" "ichiran")
}

export def prepare_ichiran_db_user_prompt [] -> string {
    return (input_default "ichiran database user" "lbr")
}

export def prepare_ichiran_db_dump_prompt [] -> string {
    return (input_default "ichiran database dump path" "./data/ichiran.pgdump")
}

# Initialises the ichiran database.
export def prepare_ichiran_db [database_name: string, database_user: string, database_dump: string] -> string {
    print $"Dropping database ($database_name)"
    exit_on_error {||
        dropdb --username=postgres --if-exists $database_name
            | complete
    }

    print "Creating database"
    exit_on_error {||
        createdb --username=postgres --owner=($database_user) --encoding='UTF8' --locale='ja_JP.utf8' --template=template0 $database_name
            | complete
    }

    print "Restoring database, this may take a while"
    warn_on_error {||
        (timeit 
            pg_restore --clean --if-exists --no-owner --role=($database_user) --username=postgres --dbname=($database_name) $database_dump
        ) | complete
    }

    print "Running ichiran commands"
    exit_on_error {||
        (timeit
            sbcl 
                --eval '(load "./data/ichiran/setup.lisp")'
                --eval '(ql:quickload :ichiran)'
                --eval '(ichiran/maintenance:add-errata)'
                --eval '(exit)'
        ) | complete
    }
}
