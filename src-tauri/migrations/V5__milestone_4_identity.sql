PRAGMA foreign_keys = OFF;

CREATE TABLE file_events_new (
    id INTEGER PRIMARY KEY,
    file_id INTEGER REFERENCES files(id) ON DELETE SET NULL,
    indexed_folder_id INTEGER NOT NULL REFERENCES indexed_folders(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL
        CHECK (event_type IN ('created', 'modified', 'deleted', 'renamed', 'moved', 'likely_renamed', 'possible_move')),
    detected_at TEXT NOT NULL,
    filesystem_time TEXT,
    old_path TEXT,
    new_path TEXT,
    confidence REAL CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
    event_source TEXT NOT NULL
        CHECK (event_source IN ('scan', 'watcher', 'reconciliation', 'manual')),
    user_confirmation TEXT
        CHECK (user_confirmation IS NULL OR user_confirmation IN ('confirmed', 'rejected')),
    user_confirmed_at TEXT
);

INSERT INTO file_events_new
    (id, file_id, indexed_folder_id, event_type, detected_at, filesystem_time,
     old_path, new_path, confidence, event_source)
SELECT id, file_id, indexed_folder_id, event_type, detected_at, filesystem_time,
       old_path, new_path, confidence, event_source
FROM file_events;

DROP TABLE file_events;
ALTER TABLE file_events_new RENAME TO file_events;

CREATE INDEX idx_file_events_timeline ON file_events (detected_at DESC, id DESC);
CREATE INDEX idx_file_events_folder_timeline ON file_events (indexed_folder_id, detected_at DESC, id DESC);
CREATE INDEX idx_file_events_type_timeline ON file_events (event_type, detected_at, id);
CREATE INDEX idx_file_events_file_history ON file_events (file_id, detected_at, id);
CREATE INDEX idx_file_events_confirmation ON file_events (user_confirmation);

CREATE TABLE file_path_history (
    id INTEGER PRIMARY KEY,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    old_path TEXT NOT NULL,
    new_path TEXT NOT NULL,
    valid_from TEXT NOT NULL,
    valid_until TEXT,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    evidence TEXT NOT NULL DEFAULT 'inference',
    event_source TEXT NOT NULL DEFAULT 'reconciliation',
    detected_at TEXT NOT NULL
);

ALTER TABLE files ADD COLUMN identity_key TEXT;
ALTER TABLE scan_file_staging ADD COLUMN identity_key TEXT;

CREATE INDEX idx_file_path_history_file
    ON file_path_history (file_id, valid_from);
CREATE INDEX idx_file_path_history_event
    ON file_path_history (detected_at);
CREATE INDEX idx_files_identity
    ON files (indexed_folder_id, identity_key);

PRAGMA foreign_keys = ON;
