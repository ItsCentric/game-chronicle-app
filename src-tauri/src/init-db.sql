CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    title TEXT,
    date TEXT DEFAULT CURRENT_TIMESTAMP,
    rating INTEGER DEFAULT 0,
    notes TEXT,
    status TEXT,
    completed BOOLEAN DEFAULT FALSE,
    minutes_played INTEGER DEFAULT 0,
    igdb_id INTEGER,
    CONSTRAINT valid_rating CHECK (rating >= 0 AND rating <= 5),
    CONSTRAINT valid_status CHECK (status IN ('backlog', 'playing', 'completed', 'dropped'))
    CONSTRAINT valid_date CHECK (date(date) IS NOT NULL)
);

CREATE TABLE IF NOT EXISTS user_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    username TEXT,
    executable_paths TEXT,
    process_monitoring_enabled INTEGER DEFAULT 0,
    process_monitoring_directory_depth INTEGER DEFAULT 3,
  	CONSTRAINT boolean_enabled CHECK (process_monitoring_enabled IN (0,1))
);

CREATE TABLE IF NOT EXISTS executable_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    executable_name TEXT,
    igdb_id INTEGER,
    minutes_played INTEGER DEFAULT 0,
    CONSTRAINT unique_executable_name UNIQUE (executable_name)
);
