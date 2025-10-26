#!/bin/nu
use common.nu *

use downloads.nu [
    dl_jmdictdb
]

# Prepares the ichiran CLI
def main [] {
    let ichiran_connection_name = prepare_ichiran_cli_connection_name_prompt
    let ichiran_connection_user = prepare_ichiran_cli_connection_user_prompt
    let ichiran_connection_password = prepare_ichiran_cli_connection_password_prompt
    let ichiran_connection_host = prepare_ichiran_cli_connection_host_prompt
    let jmdictdb_path = input_default "jmdictdb download path" "./data/jmdictdb"
    (prepare_ichiran_cli
        $ichiran_connection_name
        $ichiran_connection_user
        $ichiran_connection_password
        $ichiran_connection_host
        $jmdictdb_path
    )
}

export def prepare_ichiran_cli_connection_name_prompt []: nothing -> string {
    return (input_default "ichiran database name" "ichiran")
}

export def prepare_ichiran_cli_connection_user_prompt []: nothing -> string {
    return (input_default "ichiran database user" "lbr")
}

export def prepare_ichiran_cli_connection_password_prompt []: nothing -> string {
    return (input_password "ichiran database user password")
}

export def prepare_ichiran_cli_connection_host_prompt []: nothing -> string {
    return (input_default "ichiran database host" "localhost")
}

export def prepare_ichiran_cli [
    ichiran_connection_name: string,
    ichiran_connection_user: string,
    ichiran_connection_password: string,
    ichiran_connection_host: string,
    jmdictdb_path: string,
] {
    print "Preparing repository in ./data/ichiran"
    wget --output-document="./data/quicklisp.lisp" https://beta.quicklisp.org/quicklisp.lisp
        | complete
        | check_error
    confirm "Removing ./data/ichiran"
    rm -rf ./data/ichiran
    (sbcl
            --eval '(load "./data/quicklisp.lisp")'
            --eval '(quicklisp-quickstart:install :path "./data/ichiran")'
            --eval '(exit)'
        | complete
        | check_error
    )
    git clone --branch lbr https://github.com/Heliozoa/ichiran ./data/ichiran/local-projects/ichiran
        | complete
        | check_error

    dl_jmdictdb $jmdictdb_path

    print "Updating placeholders"
    cp ./data/ichiran/local-projects/ichiran/settings.lisp.template  ./data/ichiran/local-projects/ichiran/settings.lisp
    let ichiran_connection = $"\(\"($ichiran_connection_name)\" \"($ichiran_connection_user)\" \"($ichiran_connection_password)\" \"($ichiran_connection_host)\"\)"
    sed -i $'s#REPLACEME_CONNECTION#($ichiran_connection)#g' ./data/ichiran/local-projects/ichiran/settings.lisp
        | complete
        | check_error
    sed -i $"s#REPLACEME_DATA#($env.PWD)/data/jmdictdb/#g" ./data/ichiran/local-projects/ichiran/settings.lisp
        | complete
        | check_error

    print "Building the CLI, this may take a while"
    timeit {
        (sbcl
            --eval '(load "./data/ichiran/setup.lisp")'
            --eval '(ql:quickload :ichiran/cli)'
            --eval '(ichiran/cli:build)'
            --eval '(exit)')
        | complete
        | check_error
    }
    mv ./data/ichiran/local-projects/ichiran/ichiran-cli ./data/ichiran-cli
}
