ALTER TABLE indexed_folders ADD COLUMN normalized_path_key TEXT;
ALTER TABLE indexed_folders ADD COLUMN availability_reason TEXT NOT NULL DEFAULT 'none'
    CHECK (availability_reason IN ('none', 'missing_or_moved', 'permission_denied', 'inaccessible'));
ALTER TABLE indexed_folders ADD COLUMN last_checked_at TEXT;

UPDATE indexed_folders
SET normalized_path_key = lower(normalized_path)
WHERE normalized_path_key IS NULL;

CREATE UNIQUE INDEX idx_indexed_folders_normalized_path_key
    ON indexed_folders (normalized_path_key);

ALTER TABLE files ADD COLUMN parent_path TEXT NOT NULL DEFAULT '';

ALTER TABLE scan_runs ADD COLUMN failure_kind TEXT
    CHECK (failure_kind IS NULL OR failure_kind IN (
        'missing_or_moved', 'permission_denied', 'inaccessible', 'cancelled', 'database'
    ));

CREATE TABLE scan_file_staging (
    scan_run_id INTEGER NOT NULL REFERENCES scan_runs(id) ON DELETE CASCADE,
    indexed_folder_id INTEGER NOT NULL REFERENCES indexed_folders(id) ON DELETE CASCADE,
    normalized_path TEXT NOT NULL,
    name TEXT NOT NULL,
    parent_path TEXT NOT NULL,
    extension TEXT,
    size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
    filesystem_created_at TEXT,
    filesystem_modified_at TEXT NOT NULL,
    PRIMARY KEY (scan_run_id, normalized_path)
);

CREATE INDEX idx_scan_file_staging_folder
    ON scan_file_staging (indexed_folder_id, scan_run_id);
