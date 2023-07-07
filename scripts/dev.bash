#!/bin/bash

(
    watchexec \
        --workdir "./crates/backend" \
        --watch "./crates/backend/src" \
        --watch "./crates/api/src" \
        --watch "./crates/lbr/src" \
        --watch "./crates/core/src" \
        --debounce 10s \
        --restart \
        "cargo run --bin lbr_server" &

    cd ./crates/frontend &&
        trunk serve --address 0.0.0.0 &
)

sleep infinity
