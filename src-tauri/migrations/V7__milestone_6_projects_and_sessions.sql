CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'suggested',
    decision TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE project_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    membership_type TEXT NOT NULL DEFAULT 'suggested',
    added_at TEXT NOT NULL,
    UNIQUE(project_id, file_id)
);

CREATE INDEX idx_project_members_project_id ON project_members(project_id);
CREATE INDEX idx_project_members_file_id ON project_members(file_id);

CREATE TABLE project_suggestions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    confidence REAL NOT NULL,
    evidence TEXT NOT NULL,
    source TEXT NOT NULL,
    handled INTEGER NOT NULL DEFAULT 0,
    suggested_at TEXT NOT NULL,
    UNIQUE(project_id, file_id, source)
);

CREATE INDEX idx_project_suggestions_project_id ON project_suggestions(project_id);
CREATE INDEX idx_project_suggestions_file_id ON project_suggestions(file_id);
CREATE INDEX idx_project_suggestions_handled ON project_suggestions(handled);

CREATE TABLE activity_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    started_at TEXT NOT NULL,
    ended_at TEXT NOT NULL,
    event_summary TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'auto',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_activity_sessions_started_at ON activity_sessions(started_at);
CREATE INDEX idx_activity_sessions_project_id ON activity_sessions(project_id);

CREATE TABLE activity_session_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES activity_sessions(id) ON DELETE CASCADE,
    file_event_id INTEGER NOT NULL REFERENCES file_events(id) ON DELETE CASCADE,
    UNIQUE(session_id, file_event_id)
);

CREATE INDEX idx_activity_session_events_session_id ON activity_session_events(session_id);

CREATE TABLE activity_session_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL REFERENCES activity_sessions(id) ON DELETE CASCADE,
    file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    UNIQUE(session_id, file_id)
);

CREATE INDEX idx_activity_session_files_session_id ON activity_session_files(session_id);
CREATE INDEX idx_activity_session_files_file_id ON activity_session_files(file_id);
