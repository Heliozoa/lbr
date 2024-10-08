#!/bin/nu
use common.nu *

# Generates the Diesel schema for ichiran.
export def main [] {
    let url = input_default "ichiran database URL" "postgres://lbr:lbr@localhost/ichiran"
    generate_ichiran_schema $url
}

# Generates the Diesel schema for ichiran.
export def generate_ichiran_schema [url: string] {
    let completion = diesel print-schema --database-url $url
        | complete
        | check_error
    let path = "./crates/lbr_server/src/schema_ichiran.rs"
    echo $completion.stdout
        | save --force $path
    print $"Saved ichiran schema to ($path)"
}
