// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "deck_source_kind"))]
    pub struct DeckSourceKind;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "furigana"))]
    pub struct Furigana;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::DeckSourceKind;

    deck_sources (deck_id, source_id, kind) {
        deck_id -> Int4,
        source_id -> Int4,
        kind -> DeckSourceKind,
        threshold -> Int4,
    }
}

diesel::table! {
    decks (id) {
        id -> Int4,
        anki_deck_id -> Int8,
        name -> Text,
        user_id -> Int4,
    }
}

diesel::table! {
    ignored_words (word_id, user_id) {
        word_id -> Int4,
        user_id -> Int4,
    }
}

diesel::table! {
    kanji (id) {
        id -> Int4,
        chara -> Text,
        name -> Nullable<Text>,
        meanings -> Array<Nullable<Text>>,
        components -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    kanji_readings (id) {
        id -> Int4,
        kanji_id -> Int4,
        reading -> Text,
        okurigana -> Nullable<Text>,
    }
}

diesel::table! {
    kanji_similar (lower_kanji_id, higher_kanji_id) {
        lower_kanji_id -> Int4,
        higher_kanji_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Furigana;

    sentence_words (id) {
        id -> Int4,
        sentence_id -> Int4,
        word_id -> Nullable<Int4>,
        reading -> Nullable<Text>,
        idx_start -> Int4,
        idx_end -> Int4,
        furigana -> Array<Nullable<Furigana>>,
    }
}

diesel::table! {
    sentences (id) {
        id -> Int4,
        sentence -> Text,
        source_id -> Int4,
    }
}

diesel::table! {
    sources (id) {
        id -> Int4,
        name -> Text,
        user_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        email -> Text,
        password_hash -> Text,
        admin -> Bool,
    }
}

diesel::table! {
    word_kanji (word_id, kanji_id) {
        word_id -> Int4,
        kanji_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Furigana;

    words (id) {
        id -> Int4,
        jmdict_id -> Int4,
        word -> Text,
        reading -> Text,
        reading_standard -> Text,
        translations -> Array<Nullable<Text>>,
        furigana -> Array<Nullable<Furigana>>,
    }
}

diesel::joinable!(deck_sources -> decks (deck_id));
diesel::joinable!(deck_sources -> sources (source_id));
diesel::joinable!(decks -> users (user_id));
diesel::joinable!(ignored_words -> users (user_id));
diesel::joinable!(ignored_words -> words (word_id));
diesel::joinable!(kanji_readings -> kanji (kanji_id));
diesel::joinable!(sentence_words -> sentences (sentence_id));
diesel::joinable!(sentence_words -> words (word_id));
diesel::joinable!(sentences -> sources (source_id));
diesel::joinable!(sources -> users (user_id));
diesel::joinable!(word_kanji -> kanji (kanji_id));
diesel::joinable!(word_kanji -> words (word_id));

diesel::allow_tables_to_appear_in_same_query!(
    deck_sources,
    decks,
    ignored_words,
    kanji,
    kanji_readings,
    kanji_similar,
    sentence_words,
    sentences,
    sources,
    users,
    word_kanji,
    words,
);
