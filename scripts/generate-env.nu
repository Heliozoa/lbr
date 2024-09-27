#!/bin/nu
use common.nu *
use prepare-ichiran-db.nu [
    prepare_ichiran_db_name_prompt
    prepare_ichiran_db_user_prompt
]
use prepare-lbr-db.nu [
    prepare_lbr_db_prompt
]

# Generates the .env file.
def main [] {
    let ichiran_database_host = input_default "ichiran database host" "localhost"
    let ichiran_database_name = prepare_ichiran_db_name_prompt
    let ichiran_database_user = prepare_ichiran_db_user_prompt
    let ichiran_database_password = input_password "ichiran database"
    let private_cookie_password = input_password "cookies"
    let lbr_connection = prepare_lbr_db_prompt

    (generate_env
        $lbr_connection
        $ichiran_database_host
        $ichiran_database_name
        $ichiran_database_user
        $ichiran_database_password
        $private_cookie_password
    )
}

# Generates the .env file.
export def generate_env [
    lbr_connection: string,
    ichiran_database_host: string,
    ichiran_database_name: string,
    ichiran_database_user: string,
    ichiran_database_password: string,
    private_cookie_password: string
] {
    let ichiran_database_url = $"postgres://($ichiran_database_user):($ichiran_database_password)@($ichiran_database_host)/($ichiran_database_name)"
    [
        $"SERVER_URL=0.0.0.0:3000",
        $"DATABASE_URL=($lbr_connection)",
        $"ICHIRAN_DATABASE_URL=($ichiran_database_url)",
        $"ICHIRAN_CONNECTION_HOST=($ichiran_database_host)",
        $"ICHIRAN_CONNECTION_NAME=($ichiran_database_name)",
        $"ICHIRAN_CONNECTION_USER=($ichiran_database_user)",
        $"ICHIRAN_CONNECTION_PASSWORD=($ichiran_database_password)",
        $"ICHIRAN_CLI_PATH=./data/ichiran-cli",
        $"PRIVATE_COOKIE_PASSWORD=($private_cookie_password)",
    ]
        | str join "\n"
        | save --force .env
}
