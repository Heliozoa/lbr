// @generated automatically by Diesel CLI.

diesel::table! {
    conj_prop (id) {
        id -> Int4,
        conj_id -> Int4,
        conj_type -> Int4,
        pos -> Text,
        neg -> Nullable<Bool>,
        fml -> Nullable<Bool>,
    }
}

diesel::table! {
    conj_source_reading (id) {
        id -> Int4,
        conj_id -> Int4,
        text -> Text,
        source_text -> Text,
    }
}

diesel::table! {
    conjugation (id) {
        id -> Int4,
        seq -> Int4,
        from -> Int4,
        via -> Nullable<Int4>,
    }
}

diesel::table! {
    entry (seq) {
        seq -> Int4,
        content -> Text,
        root_p -> Bool,
        n_kanji -> Int4,
        n_kana -> Int4,
        primary_nokanji -> Bool,
    }
}

diesel::table! {
    gloss (id) {
        id -> Int4,
        sense_id -> Int4,
        text -> Text,
        ord -> Int4,
    }
}

diesel::table! {
    kana_text (id) {
        id -> Int4,
        seq -> Int4,
        text -> Text,
        ord -> Int4,
        common -> Nullable<Int4>,
        common_tags -> Text,
        conjugate_p -> Bool,
        nokanji -> Bool,
        best_kanji -> Nullable<Text>,
    }
}

diesel::table! {
    kanji (id) {
        id -> Int4,
        text -> Text,
        radical_c -> Int4,
        radical_n -> Int4,
        grade -> Nullable<Int4>,
        strokes -> Int4,
        freq -> Nullable<Int4>,
        stat_common -> Int4,
        stat_irregular -> Int4,
    }
}

diesel::table! {
    kanji_text (id) {
        id -> Int4,
        seq -> Int4,
        text -> Text,
        ord -> Int4,
        common -> Nullable<Int4>,
        common_tags -> Text,
        conjugate_p -> Bool,
        nokanji -> Bool,
        best_kana -> Nullable<Text>,
    }
}

diesel::table! {
    meaning (id) {
        id -> Int4,
        kanji_id -> Int4,
        text -> Text,
    }
}

diesel::table! {
    okurigana (id) {
        id -> Int4,
        reading_id -> Int4,
        text -> Text,
    }
}

diesel::table! {
    reading (id) {
        id -> Int4,
        kanji_id -> Int4,
        #[sql_name = "type"]
        type_ -> Text,
        text -> Text,
        suffixp -> Bool,
        prefixp -> Bool,
        stat_common -> Int4,
    }
}

diesel::table! {
    restricted_readings (id) {
        id -> Int4,
        seq -> Int4,
        reading -> Text,
        text -> Text,
    }
}

diesel::table! {
    sense (id) {
        id -> Int4,
        seq -> Int4,
        ord -> Int4,
    }
}

diesel::table! {
    sense_prop (id) {
        id -> Int4,
        tag -> Text,
        sense_id -> Int4,
        text -> Text,
        ord -> Int4,
        seq -> Int4,
    }
}

diesel::joinable!(conj_prop -> conjugation (conj_id));
diesel::joinable!(conj_source_reading -> conjugation (conj_id));
diesel::joinable!(gloss -> sense (sense_id));
diesel::joinable!(kana_text -> entry (seq));
diesel::joinable!(kanji_text -> entry (seq));
diesel::joinable!(meaning -> kanji (kanji_id));
diesel::joinable!(okurigana -> reading (reading_id));
diesel::joinable!(reading -> kanji (kanji_id));
diesel::joinable!(restricted_readings -> entry (seq));
diesel::joinable!(sense -> entry (seq));
diesel::joinable!(sense_prop -> entry (seq));
diesel::joinable!(sense_prop -> sense (sense_id));

diesel::allow_tables_to_appear_in_same_query!(
    conj_prop,
    conj_source_reading,
    conjugation,
    entry,
    gloss,
    kana_text,
    kanji,
    kanji_text,
    meaning,
    okurigana,
    reading,
    restricted_readings,
    sense,
    sense_prop,
);
