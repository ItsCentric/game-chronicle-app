ALTER TABLE games ADD description TEXT;
ALTER TABLE games RENAME COLUMN name TO title;
ALTER TABLE games ADD release_date INTEGER;

CREATE TABLE IF NOT EXISTS companies (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS genres (
    id INTEGER PRIMARY KEY NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS involved_companies (
    id INTEGER PRIMARY KEY NOT NULL,
    game_id INTEGER NOT NULL,
    company_id INTEGER NOT NULL,
    developer BOOLEAN NOT NULL,
    publisher BOOLEAN NOT NULL,
    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

CREATE TABLE IF NOT EXISTS game_time_to_beats (
    id INTEGER PRIMARY KEY NOT NULL,
    game_id INTEGER NOT NULL,
    completely INTEGER,
    normally INTEGER,
    hastily INTEGER,
    count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (game_id) REFERENCES games(id)
);

CREATE TABLE IF NOT EXISTS artworks (
    id INTEGER PRIMARY KEY NOT NULL,
    game_id INTEGER NOT NULL,
    image_id TEXT NOT NULL,
    FOREIGN KEY (game_id) REFERENCES games(id)
);

CREATE TABLE IF NOT EXISTS screenshots (
    id INTEGER PRIMARY KEY NOT NULL,
    game_id INTEGER NOT NULL,
    image_id TEXT NOT NULL,
    FOREIGN KEY (game_id) REFERENCES games(id)
);

CREATE TABLE IF NOT EXISTS videos (
    id INTEGER PRIMARY KEY NOT NULL,
    game_id INTEGER NOT NULL,
    video_id TEXT NOT NULL,
    FOREIGN KEY (game_id) REFERENCES games(id)
);

CREATE TABLE IF NOT EXISTS game_genres (
    id INTEGER PRIMARY KEY NOT NULL,
    game_id INTEGER NOT NULL,
    genre_id INTEGER NOT NULL,
    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (genre_id) REFERENCES genres(id)
);

DROP TABLE IF EXISTS games_fts;

CREATE VIRTUAL TABLE IF NOT EXISTS games_fts USING fts5(
    content_rowid="id",
    content="games",
    title,
);
