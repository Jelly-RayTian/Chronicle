PRAGMA foreign_keys = ON;

CREATE TABLE version_families (
    id INTEGER PRIMARY KEY,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL
        CHECK (status IN ('suggested', 'confirmed', 'rejected', 'superseded')),
    user_decision TEXT
        CHECK (user_decision IS NULL OR user_decision IN ('accepted', 'rejected', 'split', 'merged')),
    user_decided_at TEXT,
    merged_into_family_id INTEGER REFERENCES version_families(id) ON DELETE SET NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_version_families_status ON version_families(status);

CREATE TABLE version_family_members (
    id INTEGER PRIMARY KEY,
    version_family_id INTEGER NOT NULL REFERENCES version_families(id) ON DELETE CASCADE,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL,
    added_at TEXT NOT NULL,
    UNIQUE(version_family_id, file_id)
);

CREATE INDEX idx_version_family_members_family
    ON version_family_members(version_family_id, sort_order);
CREATE INDEX idx_version_family_members_file
    ON version_family_members(file_id);

CREATE TABLE version_family_suggestions (
    id INTEGER PRIMARY KEY,
    version_family_id INTEGER NOT NULL REFERENCES version_families(id) ON DELETE CASCADE,
    confidence REAL NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    evidence TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'heuristic'
);

CREATE INDEX idx_version_family_suggestions_family
    ON version_family_suggestions(version_family_id);
