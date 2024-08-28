#!/bin/nu
use common.nu *

def main [...args: string] {
    mut build_arg = []
    if "release" in $args {
        $build_arg = ["--build-arg="RELEASE=--release""]
    }
    docker build ...$build_arg --tag heliozoagh/lbr .
        | complete
        | check_error
}
