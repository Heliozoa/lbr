#!/bin/nu
use common.nu *

# Initialises the lbr database, resetting it in the process.
def main [] {
    initialise_logging
    let database_url = input_default "LBR database URL" "postgres://lbr:lbr@localhost/lbr"
    prepare_lbr_db $database_url
}

export def prepare_lbr_db_prompt [] -> string {
    return (input_default "LBR database URL" "postgres://lbr:lbr@localhost/lbr")
}

# Initialises the lbr database, resetting it in the process.
export def prepare_lbr_db [database_url: string] {
    confirm $"WARNING: This will reset the database at ($database_url)"
    print $"Resetting database at ($database_url)"
    diesel database reset --migration-dir ./crates/backend/migrations --database-url $database_url
        | complete
        | check_error

    print "Seeding database"
    timeit (
        cargo run --release -p lbr_server --bin update_db
            | complete
            | check_error
    )
}
