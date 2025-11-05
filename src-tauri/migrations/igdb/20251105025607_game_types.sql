CREATE TABLE IF NOT EXISTS game_types (
    id INTEGER PRIMARY KEY,
    type STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS _games (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    cover_id INTEGER,
    game_type INTEGER,
    version_parent INTEGER,
    total_rating REAL,
    FOREIGN KEY (cover_id) REFERENCES covers(id) ON DELETE SET NULL,
    FOREIGN KEY (game_type) REFERENCES game_types(id) ON DELETE SET NULL
);

INSERT INTO _games (id, name, cover_id, game_type, version_parent, total_rating) SELECT id, name, cover_id, category, version_parent, total_rating FROM games;

DROP TABLE games;
ALTER TABLE _games RENAME TO games;
CREATE VIRTUAL TABLE IF NOT EXISTS games_fts USING fts5(
    content_rowid="id",
    content="games",
    name
);
