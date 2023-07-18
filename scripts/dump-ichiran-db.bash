#!/bin/bash
# Dumps the ichiran database with pg_dump.

set -eu

url=${DATABASE_URL:-postgres://lbr:lbr@localhost/ichiran}

target="ichiran.dump"

echo "Dumping $url to $target"

pg_dump --format=t "$url" > $target
