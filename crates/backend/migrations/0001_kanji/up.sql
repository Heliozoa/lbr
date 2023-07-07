CREATE TABLE kanji (
    id SERIAL PRIMARY KEY,
    chara TEXT NOT NULL,
    name TEXT,
    meanings TEXT [] NOT NULL,
    components TEXT [] NOT NULL
);
CREATE TYPE READING_KIND AS ENUM ('onyomi', 'kunyomi');
CREATE TYPE POS AS ENUM ('prefix', 'suffix');
CREATE TABLE kanji_readings (
    id SERIAL PRIMARY KEY,
    kanji_id INTEGER NOT NULL REFERENCES kanji,
    reading TEXT NOT NULL,
    kind READING_KIND NOT NULL,
    okurigana TEXT,
    position POS
);
CREATE TABLE kanji_similar (
    lower_kanji_id INTEGER NOT NULL REFERENCES kanji,
    higher_kanji_id INTEGER NOT NULL REFERENCES kanji,
    PRIMARY KEY (lower_kanji_id, higher_kanji_id)
);