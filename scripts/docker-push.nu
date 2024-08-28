#!/bin/nu
use common.nu *

def main [] {
    docker push docker.io/heliozoagh/lbr:latest
        | complete
        | check_error
}
