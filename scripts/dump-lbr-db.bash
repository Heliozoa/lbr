#!/bin/bash
# Dumps the lbr database with pg_dump.

set -eu

url=${DATABASE_URL:-postgres://lbr:lbr@localhost/lbr}

target="lbr.dump"

echo "Dumping $url to $target"

pg_dump --format=t "$url" > $target
