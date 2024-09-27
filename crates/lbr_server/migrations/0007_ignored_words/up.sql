CREATE TABLE ignored_words (
    word_id INTEGER NOT NULL REFERENCES words,
    user_id INTEGER NOT NULL REFERENCES users,
    PRIMARY KEY (word_id, user_id)
);