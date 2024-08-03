#!/bin/nu

use common.nu *
use downloads.nu [
    dl_jmdictdb
]

# Prepares the ichiran CLI
def main [] {
    let ichiran_connection = input_default "ichiran connection string" '("ichiran" "lbr" "lbr" "localhost")'
    let jmdictdb_path = input_default "jmdictdb download path" "./data/jmdictdb"
    prepare_ichiran $ichiran_connection $jmdictdb_path
}

export def prepare_ichiran_cli_prompt [] -> string {
    return (input_default "ichiran connection string" '("ichiran" "lbr" "lbr" "localhost")')
}

export def prepare_ichiran_cli [ichiran_connection: string, jmdictdb_path: string] {
    print "Preparing repository in ./data/ichiran"
    exit_on_error {||
        wget --output-document="./data/quicklisp.lisp" https://beta.quicklisp.org/quicklisp.lisp
            | complete
    }
    confirm "Removing ./data/ichiran"
    rm -rf ./data/ichiran
    exit_on_error {||
        (sbcl
            --eval '(load "./data/quicklisp.lisp")'
            --eval '(quicklisp-quickstart:install :path "./data/ichiran")'
            --eval '(exit)'
        ) | complete
    }
    exit_on_error {||
        git clone --branch lbr https://github.com/Heliozoa/ichiran ./data/ichiran/local-projects/ichiran
            | complete
    }

    print $"Downloading jmdictdb to ($jmdictdb_path)"
    dl_jmdictdb $jmdictdb_path

    print "Updating placeholders"
    cp ./data/ichiran/local-projects/ichiran/settings.lisp.template  ./data/ichiran/local-projects/ichiran/settings.lisp
    exit_on_error {||
        sed -i $'s#REPLACEME_CONNECTION#($ichiran_connection)#g' ./data/ichiran/local-projects/ichiran/settings.lisp
            | complete
    }
    exit_on_error {||
        sed -i $"s#REPLACEME_DATA#($env.PWD)/data/jmdictdb/#g" ./data/ichiran/local-projects/ichiran/settings.lisp
            | complete
    }

    print "Building the CLI"
    exit_on_error {||
        (timeit
            sbcl
                --eval '(load "./data/ichiran/setup.lisp")'
                --eval '(ql:quickload :ichiran/cli)'
                --eval '(ichiran/cli:build)'
                --eval '(exit)'
        ) | complete
    }
    mv ./data/ichiran/local-projects/ichiran/ichiran-cli ./data/ichiran-cli
}
