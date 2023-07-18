#!/bin/bash
# Updates the ichiran schema source file.

url=${ICHIRAN_DATABASE_URL:-postgres://lbr:lbr@localhost/ichiran}
echo "Generating diesel schema from database '$url'"
if schema=$(diesel print-schema --database-url "$url"); then
    echo "$schema" > ./crates/server/src/schema_ichiran.rs
    echo "Saved diesel schema to ./crates/server/src/schema_ichiran.rs"
else
    echo "Failed to generate schema"
    exit 1
fi
