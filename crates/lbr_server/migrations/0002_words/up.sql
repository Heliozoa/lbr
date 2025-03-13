CREATE TYPE FURIGANA AS (
    word_start_idx INTEGER,
    word_end_idx INTEGER,
    reading_start_idx INTEGER,
    reading_end_idx INTEGER
);
CREATE TABLE words (
    id SERIAL PRIMARY KEY,
    jmdict_id INTEGER NOT NULL,
    word TEXT NOT NULL,
    reading TEXT NOT NULL,
    reading_standard TEXT,
    furigana FURIGANA [] NOT NULL,
    translations TEXT [] NOT NULL
);
CREATE TABLE word_kanji (
    word_id INTEGER NOT NULL REFERENCES words,
    kanji_id INTEGER NOT NULL REFERENCES kanji,
    PRIMARY KEY (word_id, kanji_id)
);