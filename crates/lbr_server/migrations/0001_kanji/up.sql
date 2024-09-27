CREATE TABLE kanji (
    id SERIAL PRIMARY KEY,
    chara TEXT NOT NULL,
    name TEXT,
    meanings TEXT [] NOT NULL,
    components TEXT [] NOT NULL
);
CREATE TABLE kanji_readings (
    id SERIAL PRIMARY KEY,
    kanji_id INTEGER NOT NULL REFERENCES kanji,
    reading TEXT NOT NULL,
    okurigana TEXT
);
CREATE TABLE kanji_similar (
    lower_kanji_id INTEGER NOT NULL REFERENCES kanji,
    higher_kanji_id INTEGER NOT NULL REFERENCES kanji,
    PRIMARY KEY (lower_kanji_id, higher_kanji_id)
);