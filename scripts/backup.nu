#!/bin/nu
use common.nu *

export def main [] {
    let today = date now | format date "%Y-%m-%d_%H-%M-%S"
    print "Backing up lbr"
    pg_dump --no-owner --dbname=postgres://lbr:lbr@localhost/lbr | save -f $"lbr_dump_($today).backup.sql"
    print "Backing up ichiran"
    pg_dump --no-owner --dbname=postgres://lbr:lbr@localhost/ichiran | save -f $"ichiran_dump_($today).backup.sql"
}
