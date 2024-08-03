use common.nu *

# Generates the .env file.
def main [] {
    let ichiran_connection = prepare_ichiran_prompt
    let ichiran_database_name = prepare_ichiran_db_name_prompt
    let ichiran_database_user = prepare_ichiran_db_user_prompt
    let ichiran_database_password = input_password "ichiran database"
    let private_cookie_password = input_password "cookies"
    let lbr_connection = prepare_lbr_db_prompt

    (generate_env
        $lbr_connection
        $ichiran_connection
        $ichiran_database_name
        $ichiran_database_user
        $ichiran_database_password
        $private_cookie_password
    )
}

# Generates the .env file.
export def generate_env [
    lbr_connection: string,
    ichiran_connection: string,
    ichiran_database_name: string,
    ichiran_database_user: string,
    ichiran_database_password: string,
    private_cookie_password: string
] {
    echo $"
SERVER_URL=0.0.0.0:3000
DATABASE_URL=($lbr_connection)
ICHIRAN_DATABASE_URL=($ichiran_connection)
ICHIRAN_CONNECTION='\(\"($ichiran_database_name)\" \"($ichiran_database_user)\" \"($ichiran_database_password)\" \"localhost\")'
ICHIRAN_CLI_PATH=./data/ichiran-cli
PRIVATE_COOKIE_PASSWORD=($private_cookie_password)
" | save .env
}
