#!/bin/nu

use common.nu *

# Functions for downloading data files used by the project.
def main [] {
    let ichiran_dump_path = input_default "Ichiran dump download path" "./data/ichiran.pgdump"
    let jmdict_path = input_default "JMdict download path" "./data/JMdict_e_examp.xml"
    let jmdictdb_path = input_default "JMdict download path" "./data/jmdictdb"
    let kanjidic_path = input_default "KANJIDIC2 download path" "./data/kanjidic2.xml"
    let kradfile_path = input_default "KRADFILE download path" "./data/kradfile"
    let jmdict_furigana_path = input_default "JmdictFurigana download path" "./data/JmdictFurigana.json"

    dl_ichiran_dump $ichiran_dump_path
    dl_jmdict $jmdict_path
    dl_jmdictdb $jmdictdb_path
    dl_kanjidic $kanjidic_path
    dl_kradfile $kradfile_path
    dl_jmdict_furigana $jmdict_furigana_path
}

# Downloads the database dump that is used to initialise the ichiran database.
export def dl_ichiran_dump [path: string] {
    print $"Downloading ichiran dump to ($path)"
    exit_on_error {||
        curl "https://api.github.com/repos/tshatrov/ichiran/releases/latest"
            | jq '.assets[0].browser_download_url'
            | str trim --char '"'
            | wget --output-document=-
            | save --force $path
    }
}

# Downloads JMdict, a Japanese-English dictionary that serves as the basis for the Japanese data used by the project.
export def dl_jmdict [path: string] {
    print $"Downloading JMdict to ($path)"
    exit_on_error {||
        wget --output-document=- http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz \
            | gunzip --stdout -
            | save --force $path
    }
}

# Downloads JMdictDB, a database that contains JMdict's data for use by ichiran.
export def dl_jmdictdb [path: string] {
    print $"Downloading jmdictdb to ($path)"
    exit_on_error {||
        wget --output-document=- https://gitlab.com/yamagoya/jmdictdb/-/archive/master/jmdictdb-master.tar.gz?path=jmdictdb/data
            | tar zxf - --overwrite --strip-components=3 --directory=($path) jmdictdb-master-jmdictdb-data/jmdictdb/data/
    }
}

# Downloads KANJIDIC2, which contains information about kanji.
export def dl_kanjidic [path: string] {
    print $"Downloading KANJIDIC2 to ($path)"
    exit_on_error {||
        wget --output-document=- http://www.edrdg.org/kanjidic/kanjidic2.xml.gz \
            | gunzip --stdout -
            | save --force $path
    }
}

# Downloads KRADFILE, which contains info about kanji components.
export def dl_kradfile [path: string] {
    print $"Downloading KRADFILE to ($path)"
    exit_on_error {||
        wget --output-document=- http://ftp.edrdg.org/pub/Nihongo/kradfile.gz \
            | gunzip -c -
            | save --force $path
    }
}

# Downloads JmdictFurigana, which contains furigana information for JMdict words.
export def dl_jmdict_furigana [path: string] {
    print $"Downloading JmdictFurigana to ($path)"
    curl https://api.github.com/repos/Doublevil/JmdictFurigana/releases/latest
        | jq '.assets[] | select(.name == "JmdictFurigana.json").browser_download_url'
        | wget --output-document=- $in
        | jq .
        | save --force $path
}
