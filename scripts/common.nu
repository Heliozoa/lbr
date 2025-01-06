#!/bin/nu
# Utility functions for the other scripts

export def env-vars []: nothing -> record {
    open .env  
        |lines 
        | split column '#'
        | get column1 
        | parse "{key}={value}"
        | str trim value --char '"'
        | transpose --header-row --as-record
}

export def initialise_logging [] {
    $env.RUST_LOG = "info"
}

export def input_default [prompt: string, default: string]: nothing -> string {
    let inp = input $"($prompt) \(default `($default)`): "
    if ($inp | is-empty) {
        return $default
    } else {
        return $inp
    }
}

export def input_password [target: string]: nothing -> string {
    let inp = input --suppress-output $"Input password for ($target): "
    if ($inp | is-empty) {
        print "Password cannot be empty"
        exit 1
    } else {
        print ""
        return $inp
    }
}

export def confirm [prompt: string] {
    let confirm = input $"($prompt). Enter 'y' to continue: "
    if $confirm != 'y' {
        print "Cancelling operation"
        exit 0
    }
}

export def check_warning []: record -> string {
    if $in.exit_code != 0 {
        print "Error running external command"
        print $"stdout: ($in.stdout)"
        print $"stderr: ($in.stderr)"
    }
    return $in.stdout
}

export def check_error []: record -> string {
    if $in.exit_code != 0 {
        print "Error running external command"
        print $"stdout: ($in.stdout)"
        print $"stderr: ($in.stderr)"
        exit 1
    } else {
        return $in.stdout
    }
}
