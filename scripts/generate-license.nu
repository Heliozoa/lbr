#!/bin/nu
use common.nu *

# Generates files that contain all of the third-party license information.
def main [] {
    initialise_logging
    let target = input_default "Generate license for the website (`web`) or docker (`docker`)?" "web"
    generate_license $target
}

# Generates files that contain all of the third-party license information.
export def generate_license [target: string]: nothing -> string {
    if ($target == "web") {
        cargo about generate ./about/web.hbs
            | complete
            | check_error
            | save -f ./data/license-web.html
    } else if ($target == "docker") {
        cargo about generate ./about/docker.hbs
            | complete
            | check_error
            | save -f ./data/license-docker.md
    } else {
        print "Invalid input"
        exit 1
    }
}
