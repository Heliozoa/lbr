#!/bin/bash
# Initialises the LBR database.

url=${DATABASE_URL:-postgres://lbr:lbr@localhost/lbr}
read -p "WARNING: This will reset the database at $url. Enter 'y' to continue.
" -r input
if [ "$input" != "y" ]; then
    exit 0
fi

echo "Resetting database at '$url'"
if ! diesel database reset --migration-dir ./crates/server/migrations --database-url "$url"; then
    echo "Failed to reset database"
    exit 1
fi

echo "Seeding database kanji"
cargo run -p lbr_server --bin init_db_kanji
echo "Seeding database words"
cargo run -p lbr_server --bin init_db_words

echo "Finished"
