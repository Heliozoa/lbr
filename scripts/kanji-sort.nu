def main [] {
    sort "kanji_extra"
    sort "kanji_names"
    sort "kanji_similar"
}

def sort [name: string] {
    let sorted = jq --sort-keys . $"./crates/jadata/data/($name).json"
    $sorted | save -f $"./crates/jadata/data/($name).json"
}