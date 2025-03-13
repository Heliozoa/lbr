#!/bin/nu
use common.nu *

# Updates the ichiran seq => word id database info.
export def main [] {
    initialise_logging
    timeit {
        cargo run --release --bin init_ichiran_word_to_id
            | complete
            | check_error
    }
}
