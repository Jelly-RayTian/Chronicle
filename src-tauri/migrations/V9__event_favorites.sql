CREATE TABLE favorite_events (
    event_id INTEGER NOT NULL PRIMARY KEY REFERENCES file_events(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL
);
