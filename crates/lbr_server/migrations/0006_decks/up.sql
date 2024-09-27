CREATE TABLE decks (
    id SERIAL PRIMARY KEY,
    anki_deck_id BIGINT NOT NULL,
    name TEXT NOT NULL,
    user_id INTEGER NOT NULL REFERENCES users,
    UNIQUE (name, user_id)
);
CREATE TYPE DECK_SOURCE_KIND AS ENUM ('word', 'kanji');
CREATE TABLE deck_sources (
    deck_id INTEGER NOT NULL REFERENCES decks,
    source_id INTEGER NOT NULL REFERENCES sources,
    kind DECK_SOURCE_KIND NOT NULL,
    threshold INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (deck_id, source_id, kind)
);