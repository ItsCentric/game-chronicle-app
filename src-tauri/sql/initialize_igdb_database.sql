CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    cover_id INTEGER,
    category INTEGER,
    version_parent INTEGER,
    total_rating REAL,
    FOREIGN KEY (cover_id) REFERENCES covers(id) ON DELETE SET NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS games_fts USING fts5(
    content_rowid="id",
    content="games",
    name
);

CREATE TABLE IF NOT EXISTS covers (
    id INTEGER PRIMARY KEY,
    image_id TEXT
);

CREATE TABLE IF NOT EXISTS websites (
    id INTEGER PRIMARY KEY,
    url TEXT
);

CREATE TABLE IF NOT EXISTS game_websites (
    game_id INTEGER,
    website_id INTEGER,
    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (website_id) REFERENCES websites(id),
    PRIMARY KEY (game_id, website_id)
);

CREATE TABLE IF NOT EXISTS platforms (
    id INTEGER PRIMARY KEY,
    name TEXT,
    category INTEGER
);

CREATE TABLE IF NOT EXISTS similar_games (
    game_id INTEGER,
    similar_game_id INTEGER,
    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (similar_game_id) REFERENCES games(id),
    PRIMARY KEY (game_id, similar_game_id)
);

CREATE TABLE IF NOT EXISTS game_platforms (
    game_id INTEGER,
    platform_id INTEGER,
    FOREIGN KEY (game_id) REFERENCES games(id),
    FOREIGN KEY (platform_id) REFERENCES platforms(id),
    PRIMARY KEY (game_id, platform_id)
);
