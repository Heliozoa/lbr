#!/bin/bash
# Generates the kanjifile and wordfile using jadata.

echo "Generating kanjifile"
cargo run -p jadata -- \
    kanjifile \
    --kanjidic ./crates/jadata/external/kanjidic2.xml \
    --kradfile ./crates/jadata/external/kradfile \
    --names ./crates/jadata/included/kanjifile_names.json \
    --similar ./crates/jadata/included/kanjifile_similar.json \
    --manual ./crates/jadata/included/kanjifile_manual.json \
    --output ./crates/jadata/generated/kanjifile.json

echo "Generating wordfile"
cargo run -p jadata -- \
    wordfile \
    --jmdict ./crates/jadata/external/JMdict_e_examp.xml \
    --jmdict-version "$(sed -n '0,/<!-- Rev \([0-9.]*\)/s//\1/p' ./crates/jadata/external/JMdict_e_examp.xml)" \
    --furigana ./crates/jadata/external/JmdictFurigana.json \
    --output ./crates/jadata/generated/wordfile.json

echo "Finished"
