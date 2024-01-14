CREATE TABLE words (
    id SERIAL PRIMARY KEY,
    jmdict_id INTEGER,
    translations TEXT [] NOT NULL
);
CREATE TABLE written_forms (
    id SERIAL PRIMARY KEY,
    word_id INTEGER NOT NULL REFERENCES words,
    written_form TEXT NOT NULL
);
CREATE TABLE word_kanji (
    written_form_id INTEGER NOT NULL REFERENCES written_forms,
    kanji_id INTEGER NOT NULL REFERENCES kanji,
    PRIMARY KEY (written_form_id, kanji_id)
);
CREATE TYPE FURIGANA AS (
    word_start_idx INTEGER,
    word_end_idx INTEGER,
    reading_start_idx INTEGER,
    reading_end_idx INTEGER
);
CREATE TABLE word_readings (
    id SERIAL PRIMARY KEY,
    written_form_id INTEGER NOT NULL REFERENCES written_forms,
    reading TEXT NOT NULL,
    reading_katakana TEXT NOT NULL,
    furigana FURIGANA [] NOT NULL,
    usually_kana BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE TABLE word_ichiran (
    id SERIAL PRIMARY KEY,
    root_seq INTEGER NOT NULL,
    ichiran_seq INTEGER NOT NULL
);