CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    cover_id INTEGER,
    FOREIGN KEY (cover_id) REFERENCES covers(id)
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
