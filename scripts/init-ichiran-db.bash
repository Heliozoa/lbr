#!/bin/bash
# Initialises the ichiran database from the dump.

DBNAME=ichiran
DUMP="./data/ichiran.pgdump"

read -p "WARNING: This will reset the '$DBNAME' database and restore it from $DUMP. Enter 'y' to continue
" -r input
if [ "$input" != "y" ]; then
    exit 0
fi

echo "Dropping database"
if ! dropdb --username=postgres --if-exists "$DBNAME"; then
    echo "Error dropping database"
    exit 1
fi
echo "Creating database"
if ! createdb --username=postgres --owner=lbr --encoding='UTF8' --locale='ja_JP.utf8' --template=template0 "$DBNAME"; then
    echo "Error creating database"
    exit 1
fi
echo "Restoring database"
if ! pg_restore --clean --if-exists --no-owner --role=lbr --username=postgres --dbname="$DBNAME" "$DUMP"; then
    echo "Errors restoring database, but these are probably fine to ignore"
    exit 1
fi

echo "Finished"