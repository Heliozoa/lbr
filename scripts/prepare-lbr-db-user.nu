#!/bin/nu

use common.nu *

# Creates the lbr database user if missing.
export def main [] {
    print "Creating database user `lbr`, ignoring errors"
    psql --user postgres --command "CREATE ROLE lbr WITH LOGIN CREATEDB PASSWORD 'lbr';"
}
