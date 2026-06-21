PRAGMA foreign_keys = ON;

CREATE TABLE indexed_folders (
    id INTEGER PRIMARY KEY,
    normalized_path TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    added_at TEXT NOT NULL,
    last_successful_scan_at TEXT,
    monitoring_enabled INTEGER NOT NULL DEFAULT 0 CHECK (monitoring_enabled IN (0, 1)),
    availability_status TEXT NOT NULL DEFAULT 'available'
        CHECK (availability_status IN ('available', 'missing', 'unavailable'))
);

CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    indexed_folder_id INTEGER NOT NULL REFERENCES indexed_folders(id) ON DELETE CASCADE,
    normalized_path TEXT NOT NULL,
    name TEXT NOT NULL,
    extension TEXT,
    size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
    filesystem_created_at TEXT,
    filesystem_modified_at TEXT NOT NULL,
    first_indexed_at TEXT NOT NULL,
    last_seen_at TEXT NOT NULL,
    is_present INTEGER NOT NULL DEFAULT 1 CHECK (is_present IN (0, 1)),
    UNIQUE (indexed_folder_id, normalized_path)
);

CREATE TABLE file_events (
    id INTEGER PRIMARY KEY,
    file_id INTEGER REFERENCES files(id) ON DELETE SET NULL,
    indexed_folder_id INTEGER NOT NULL REFERENCES indexed_folders(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL
        CHECK (event_type IN ('created', 'modified', 'deleted', 'renamed', 'moved')),
    detected_at TEXT NOT NULL,
    filesystem_time TEXT,
    old_path TEXT,
    new_path TEXT,
    confidence REAL CHECK (confidence IS NULL OR (confidence >= 0.0 AND confidence <= 1.0)),
    event_source TEXT NOT NULL
        CHECK (event_source IN ('scan', 'watcher', 'reconciliation', 'manual'))
);

CREATE TABLE scan_runs (
    id INTEGER PRIMARY KEY,
    indexed_folder_id INTEGER NOT NULL REFERENCES indexed_folders(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    status TEXT NOT NULL CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
    files_seen INTEGER NOT NULL DEFAULT 0 CHECK (files_seen >= 0),
    warning_count INTEGER NOT NULL DEFAULT 0 CHECK (warning_count >= 0),
    error_count INTEGER NOT NULL DEFAULT 0 CHECK (error_count >= 0)
);

CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX idx_file_events_timeline
    ON file_events (detected_at DESC, id DESC);
CREATE INDEX idx_file_events_folder_timeline
    ON file_events (indexed_folder_id, detected_at DESC, id DESC);
CREATE INDEX idx_scan_runs_folder_started
    ON scan_runs (indexed_folder_id, started_at DESC, id DESC);

