#!/bin/nu
use common.nu *

# Starts the project up.
def --wrapped main [...args] {
    initialise_logging
    cargo leptos watch ...$args
}
