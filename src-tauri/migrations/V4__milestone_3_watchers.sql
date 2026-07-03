CREATE TABLE watcher_states (
    indexed_folder_id INTEGER PRIMARY KEY REFERENCES indexed_folders(id) ON DELETE CASCADE,
    desired_state TEXT NOT NULL DEFAULT 'disabled'
        CHECK (desired_state IN ('disabled', 'enabled', 'paused')),
    runtime_state TEXT NOT NULL DEFAULT 'stopped'
        CHECK (runtime_state IN ('stopped', 'running', 'paused', 'unavailable', 'error')),
    coalescing_window_ms INTEGER NOT NULL DEFAULT 750
        CHECK (coalescing_window_ms >= 100 AND coalescing_window_ms <= 60000),
    last_started_at TEXT,
    last_stopped_at TEXT,
    last_event_at TEXT,
    last_error_at TEXT,
    last_error_kind TEXT,
    last_error_message TEXT,
    events_recorded INTEGER NOT NULL DEFAULT 0 CHECK (events_recorded >= 0),
    events_dropped INTEGER NOT NULL DEFAULT 0 CHECK (events_dropped >= 0)
);

INSERT INTO watcher_states (indexed_folder_id, desired_state, runtime_state)
SELECT id,
       CASE WHEN monitoring_enabled = 1 THEN 'enabled' ELSE 'disabled' END,
       'stopped'
FROM indexed_folders;

CREATE INDEX idx_watcher_states_desired
    ON watcher_states (desired_state, runtime_state);
