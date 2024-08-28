#!/bin/nu
use common.nu *

use downloads.nu [
    dl_jmdictdb
]

# Prepares the ichiran CLI
def main [] {
    let ichiran_database_name = input_default "ichiran database name" "ichiran"
    let ichiran_database_user = input_default "ichiran database user" "lbr"
    let ichiran_database_password = input_password "ichiran database password"
    let ichiran_database_host = input_default "ichiran database host" "localhost"
    let jmdictdb_path = input_default "jmdictdb download path" "./data/jmdictdb"
    (prepare_ichiran_cli
        $ichiran_database_name
        $ichiran_database_user
        $ichiran_database_password
        $ichiran_database_host
        $jmdictdb_path
    )
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
    timeit (
        sbcl
            --eval '(load "./data/ichiran/setup.lisp")'
            --eval '(ql:quickload :ichiran/cli)'
            --eval '(ichiran/cli:build)'
            --eval '(exit)'
        | complete
        | check_error
    )
    mv ./data/ichiran/local-projects/ichiran/ichiran-cli ./data/ichiran-cli
}
