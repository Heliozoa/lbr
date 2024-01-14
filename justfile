set shell := ["bash", "-uc"]


# Lists the available just commands
_default:
    just --list --unsorted


# Starts the dev server and watches for changes
watch:
    cargo leptos watch


# Generates the license information for the given target (targets: ["web", "docker"])
generate-license target="web" overwrite="true":
    #!/usr/bin/env bash
    set -euo pipefail

    if [ {{ target }} = "web" ]; then
        if [ ! -f ./data/license.html ] || [ {{overwrite}} = "true" ]; then
            cargo about generate ./about/web.hbs > ./data/license-web.html
        fi
    elif [ {{ target }} = "docker" ]; then
        if [ ! -f ./data/license.md ] || [ {{overwrite}} = "true" ]; then
            cargo about generate ./about/docker.hbs > ./data/license-docker.md
        fi
    else
        echo "Unexpected target '{{target}}'"
        exit 1
    fi


# Sets up local databases and downloads and generates files required for local development
prepare-repository: && (generate-license "web" "false") prepare-data generate-jadata prepare-ichiran prepare-db-user prepare-ichiran-db prepare-lbr-db build-cli build-cli-docker
    cp ./example.env ./.env


# Prepares ichiran's settings.lisp file
prepare-ichiran-settings ichiran-connection='(\"ichiran\" \"lbr\" \"lbr\" \"localhost\")':
    cp ./data/ichiran/local-projects/ichiran/settings.lisp.template  ./data/ichiran/local-projects/ichiran/settings.lisp
    sed -i 's#REPLACEME_CONNECTION#{{ichiran-connection}}#g' ./data/ichiran/local-projects/ichiran/settings.lisp
    sed -i "s#REPLACEME_DATA#$(pwd)/data/jmdictdb/#g" ./data/ichiran/local-projects/ichiran/settings.lisp


# Prepares the ichiran repo
prepare-ichiran ichiran-connection='(\"ichiran\" \"lbr\" \"lbr\" \"localhost\")': (prepare-ichiran-settings ichiran-connection)
    wget --output-document="./data/quicklisp.lisp" https://beta.quicklisp.org/quicklisp.lisp
    rm -rf ./data/ichiran
    sbcl \
        --eval '(load "./data/quicklisp.lisp")' \
        --eval '(quicklisp-quickstart:install :path "./data/ichiran")' \
        --eval '(exit)'
    git clone --branch lbr https://github.com/Heliozoa/ichiran ./data/ichiran/local-projects/ichiran


# Builds ichiran-cli for local use
build-cli ichiran-connection='(\"ichiran\" \"lbr\" \"lbr\" \"localhost\")': (prepare-ichiran-settings ichiran-connection)
    sbcl \
        --eval '(load "./data/ichiran/setup.lisp")' \
        --eval '(ql:quickload :ichiran/cli)' \
        --eval '(ichiran/cli:build)' \
        --eval '(exit)'
    mv ./data/ichiran/local-projects/ichiran/ichiran-cli ./data/ichiran-cli


# Builds ichiran-cli for Docker
build-cli-docker ichiran-connection='(\"ichiran\" \"lbr\" \"lbr\" \"localhost\")': (prepare-ichiran-settings ichiran-connection)
    sbcl \
        --eval '(load "./data/ichiran/setup.lisp")' \
        --eval '(ql:quickload :ichiran/cli)' \
        --eval '(ichiran/cli:build)' \
        --eval '(exit)'
    mv ./data/ichiran/local-projects/ichiran/ichiran-cli ./data/ichiran-cli-docker




# #### DATABASE COMMANDS ####
DATABASE-COMMANDS:


# Connects to the database with psql
psql database-url="postgres://lbr:lbr@localhost/lbr":
    psql {{database-url}}


# Creates the postgres user 'lbr' with the password 'lbr'
prepare-db-user:
    echo "Creating database user, ignoring errors"
    -psql --user postgres --command "CREATE ROLE lbr WITH LOGIN CREATEDB PASSWORD 'lbr';"


# Sets up the local lbr database
prepare-lbr-db database-url="postgres://lbr:lbr@localhost/lbr":
    #!/usr/bin/env bash
    set -euo pipefail
    
    export RUST_LOG=info

    url=${DATABASE_URL:-{{database-url}}}
    read -p "WARNING: This will reset the database at $url. Enter 'y' to continue.
    " -r input
    if [ "$input" != "y" ]; then
        exit 0
    fi

    echo "Resetting database at '$url'"
    if ! diesel database reset --migration-dir ./crates/backend/migrations --database-url "$url"; then
        echo "Failed to reset database"
        exit 1
    fi

    echo "Seeding database kanji"
    cargo run --release -p lbr_server --bin init_db_kanji
    echo "Seeding database words"
    cargo run --release -p lbr_server --bin init_db_words

    echo "Finished"


# Downloads the latest ichiran database dump. Set `force` to overwrite existing files
dl-ichiran-dump force="false":
    #!/usr/bin/env bash
    set -euo pipefail

    path=./data/ichiran.pgdump
    echo "Checking ${path}"
    if [ ! -f ${path} ] || [ ! {{force}} = "false" ]; then
        curl https://api.github.com/repos/tshatrov/ichiran/releases/latest \
            | jq '.assets[0].browser_download_url' \
            | xargs wget --output-document=${path}
    fi


# Sets up the local ichiran database
prepare-ichiran-db database-name="ichiran" dump="./data/ichiran.pgdump" ichiran-connection='(\"ichiran\" \"lbr\" \"lbr\" \"localhost\")': dl-ichiran-dump (prepare-ichiran-settings ichiran-connection)
    #!/usr/bin/env bash
    set -euo pipefail

    read -p "WARNING: This will reset the '{{database-name}}' database and restore it from {{dump}}. Enter 'y' to continue
    " -r input
    if [ "$input" != "y" ]; then
        exit 0
    fi

    echo "Dropping database"
    if ! dropdb --username=postgres --if-exists "{{database-name}}"; then
        echo "Error dropping database"
        exit 1
    fi
    echo "Creating database"
    if ! createdb --username=postgres --owner=lbr --encoding='UTF8' --locale='ja_JP.utf8' --template=template0 "{{database-name}}"; then
        echo "Error creating database"
        exit 1
    fi
    echo "Restoring database"
    if ! pg_restore --clean --if-exists --no-owner --role=lbr --username=postgres --dbname="{{database-name}}" "{{dump}}"; then
        echo "Errors restoring database, but these are probably fine to ignore"
    fi

    sbcl \
        --eval '(load "./data/ichiran/setup.lisp")' \
        --eval '(ql:quickload :ichiran)' \
        --eval '(ichiran/maintenance:add-errata)' \
        --eval '(exit)'

    echo "Finished"


# Generates the ichiran schema Rust file
generate-ichiran-schema url="postgres://lbr:lbr@localhost/ichiran":
    #!/usr/bin/env bash
    set -euo pipefail

    url=${ICHIRAN_DATABASE_URL:-{{url}}}
    echo "Generating diesel schema from database '$url'"
    if schema=$(diesel print-schema --database-url "$url"); then
        echo "$schema" > ./crates/backend/src/schema_ichiran.rs
        echo "Saved diesel schema to ./crates/backend/src/schema_ichiran.rs"
    else
        echo "Failed to generate schema"
        exit 1
    fi


# Creates a pg_dump of the lbr database
dump-lbr-db url="postgres://lbr:lbr@localhost/lbr" target="lbr.dump":
    #!/usr/bin/env bash
    set -euo pipefail

    url=${DATABASE_URL:-{{url}}}
    echo "Dumping ${url} to {{target}}"
    pg_dump --format=t "${url}" > {{target}}


# Creates a pg_dump of the ichiran database
dump-ichiran-db url="postgres://lbr:lbr@localhost/ichiran" target="ichiran.dump":
    #!/usr/bin/env bash
    set -euo pipefail

    url=${DATABASE_URL:-{{url}}}
    echo "Dumping ${url} to {{target}}"
    pg_dump --format=t "${url}" > {{target}}




# #### DATA FILE COMMANDS ####
DATA-FILE-COMMANDS:


# Downloads data files required to run the project. Set `force` to overwrite existing files
prepare-data: dl-jmdictdb dl-jmdict dl-kanjidic dl-kradfile dl-furigana


# Downloads jmdictdb. Set `force` to overwrite existing files
dl-jmdictdb force="false":
    #!/usr/bin/env bash
    set -euo pipefail

    path=./data/jmdictdb
    echo "Checking ${path}"
    if [ ! -f ${path} ] || [ ! {{force}} = "false" ]; then
        rm -rf ${path}
        mkdir -p ${path}
        echo "Downloading jmdictdb"
        wget --output-document=- https://gitlab.com/yamagoya/jmdictdb/-/archive/master/jmdictdb-master.tar.gz?path=jmdictdb/data \
            | tar zxf - --strip-components=3 --directory=${path} jmdictdb-master-jmdictdb-data/jmdictdb/data/
    else
        echo "Already exists"
    fi


# Downloads JMdict_e_examp.xml. Set `force` to overwrite existing files
dl-jmdict force="false":
    #!/usr/bin/env bash
    set -euo pipefail

    dir=./data/jadata/external
    name=JMdict_e_examp.xml
    path=${dir}/${name}
    echo "Checking ${path}"
    if [ ! -f  ${path} ] || [ ! {{force}} = "false" ]; then
        rm -rf ${path}
        mkdir -p ${dir}
        echo "Downloading JMdict_e_examp"
        wget --output-document=- http://ftp.edrdg.org/pub/Nihongo/JMdict_e_examp.gz \
            | gunzip -c - > ${path}
    else
        echo "Already exists"
    fi


# Downloads kanjidic2.xml. Set `force` to overwrite existing files
dl-kanjidic force="false":
    #!/usr/bin/env bash
    set -euo pipefail

    dir=./data/jadata/external
    name=kanjidic2.xml
    path=${dir}/${name}
    echo "Checking ${path}"
    if [ ! -f ${path} ] || [ ! {{force}} = "false" ]; then
        rm -rf ${path}
        mkdir -p ${dir}
        echo "Downloading kanjidic"
        wget --output-document=- http://www.edrdg.org/kanjidic/kanjidic2.xml.gz \
            | gunzip -c - > ${path}
    else
        echo "Already exists"
    fi


# Downloads kradfile. Set `force` to overwrite existing files
dl-kradfile force="false":
    #!/usr/bin/env bash
    set -euo pipefail

    dir=./data/jadata/external
    name=kradfile
    path=${dir}/${name}
    echo "Checking ${path}"
    if [ ! -f ${path} ] || [ ! {{force}} = "false" ]; then
        rm -rf ${path}
        mkdir -p ${dir}
        echo "Downloading kradfile"
        wget --output-document=- http://ftp.edrdg.org/pub/Nihongo/kradfile.gz \
            | gunzip -c - > ${path}
    else
        echo "Already exists"
    fi


# Downloads JmdictFurigana.json. Set `force` to overwrite existing files
dl-furigana force="false":
    #!/usr/bin/env bash
    set -euo pipefail

    dir=./data/jadata/external
    name=JmdictFuriganaPretty.json
    path=${dir}/${name}
    echo "Checking ${path}"
    if [ ! -f ${path} ] || [ ! {{force}} = "false" ]; then
        rm -rf ${path}
        mkdir -p ${dir}
        echo "Downloading JmdictFurigana"
        curl https://api.github.com/repos/Doublevil/JmdictFurigana/releases/latest \
            | jq '.assets[] | select(.name == "JmdictFurigana.json").browser_download_url' \
            | xargs wget --output-document=- \
            | jq . > ${path}
    else
        echo "Already exists"
    fi


# Generates the kanjifile and wordfile
generate-jadata:
    #!/usr/bin/env bash
    set -euo pipefail

    if [ ! -d ./data/jadata ]; then
        git clone https://github.com/Heliozoa/jadata ./data/jadata
    else
        cd ./data/jadata && git pull
        cd ../../
    fi

    cd ./data/jadata

    export RUST_LOG=info

    echo "Generating wordfile"
    cargo run --release -- \
        kanjifile \
        -v 1 \
        -d ./external/kanjidic2.xml \
        -k ./external/kradfile \
        -s ./included/kanjifile_skeleton.json \
        -t json \
        -o ./generated/kanjifile.json

    echo "Generating wordfile"
    cargo run --release -- \
        wordfile \
        -v 1 \
        -j ./external/JMdict_e_examp.xml \
        -f ./external/JmdictFuriganaPretty.json \
        -s ./included/wordfile_skeleton.json \
        -t json \
        -o ./generated/wordfile.json

    echo "Finished"




# #### DOCKER COMMANDS ####
DOCKER-COMMANDS:


# Builds the Docker image
docker-build release="--release":
    docker build --build-arg="RELEASE={{ release }}" --tag heliozoagh/lbr .


# Runs the Docker image
docker-run database-url="postgres://lbr:lbr@localhost/lbr" ichiran-database-url="postgres://lbr:lbr@localhost/ichiran" ichiran-connection='(\"ichiran\" \"lbr\" \"lbr\" \"localhost\")' private-cookie-password="uvoo4rei1aiN0po4aitix9pie0eo7aaZei0aem6ix5oi5quooxaiQuooTohs2Pha":
    docker run \
        --init \
        --rm \
        --env DATABASE_URL="{{ database-url }}" \
        --env ICHIRAN_DATABASE_URL="{{ ichiran-database-url }}" \
        --env ICHIRAN_CONNECTION="{{ ichiran-connection }}" \
        --env PRIVATE_COOKIE_PASSWORD="{{ private-cookie-password }}" \
        --network=host \
        heliozoagh/lbr


# Pushes the Docker image
docker-push: docker-build
    docker push docker.io/heliozoagh/lbr:latest
