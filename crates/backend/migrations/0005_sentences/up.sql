CREATE TABLE sentences (
    id SERIAL PRIMARY KEY,
    sentence TEXT UNIQUE NOT NULL,
    source_id INTEGER NOT NULL REFERENCES sources
);
CREATE TABLE sentence_words (
    id SERIAL PRIMARY KEY,
    sentence_id INTEGER NOT NULL REFERENCES sentences,
    word_id INTEGER NOT NULL REFERENCES words,
    reading TEXT,
    idx_start INTEGER NOT NULL,
    idx_end INTEGER NOT NULL,
    furigana FURIGANA [] NOT NULL
);