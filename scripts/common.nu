#!/bin/nu
# Utility functions for the other scripts

export def env-vars [] -> record {
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

export def input_default [prompt: string, default: string] -> string {
    let inp = input $"($prompt) \(default `($default)`): "
    if ($inp | is-empty) {
        return $default
    } else {
        return $inp
    }
}

export def input_password [target: string] -> string {
    let inp = input --suppress-output $"Input password for ($target): "
    if ($inp | is-empty) {
        print "Password cannot be empty"
        exit 1
    } else {
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

export def exit_on_error [cmd: closure] -> record {
    let completion: record = warn_on_error $cmd
    if ($completion.exit_code != 0) {
        exit 1
    }
    return $completion
}

export def warn_on_error [cmd: closure] -> record {
    let completion: record = do $cmd
    if ($completion.exit_code != 0) {
        print "Error(s) running command"
        print $completion.stderr
    }
    return $completion
}
