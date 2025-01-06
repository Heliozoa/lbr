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
    if (target == "web") {
        cargo about generate ./about/web.hbs > ./data/license-web.html
            | complete
            | check_error
    } else if (target == "docker") {
        cargo about generate ./about/docker.hbs > ./data/license-docker.md
            | complete
            | check_error
    } else {
        print "Invalid input"
        exit 1
    }
}
