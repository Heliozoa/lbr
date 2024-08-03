#!/bin/nu

use common.nu *

# Updates the ichiran seq => word id database info.
export def main [] {
    initialise_logging
    exit_on_error {||
        (timeit
            cargo run --bin init_ichiran_seq_to_word_id
        ) | complete
    }
}
