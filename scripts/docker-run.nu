#!/bin/nu
use common.nu *

def main [...args: string] {
    mut init = ["--init"]
    mut bash = []
    if "bash" in $args {
        $init = []
        $bash = ["bash", "-c", "apt", "install", "-y", "wget", "&&", "wget", "https://truststore.pki.rds.amazonaws.com/global/global-bundle.pem", "&&", "cp", "./global-bundle.pem", "/usr/local/share/ca-certificates", "&&", "update-ca-certificates"]
    }

    (docker run
        ...$init
        --rm
        --env DATABASE_URL="postgres://lbr:lbr@localhost/lbr"
        --env ICHIRAN_DATABASE_URL="postgres://lbr:lbr@localhost/ichiran"
        --env ICHIRAN_CONNECTION='("ichiran" "lbr" "lbr" "localhost")'
        --env PRIVATE_COOKIE_PASSWORD="uvoo4rei1aiN0po4aitix9pie0eo7aaZei0aem6ix5oi5quooxaiQuooTohs2Pha"
        --network=host
        heliozoagh/lbr
        ...$bash
    )
}
