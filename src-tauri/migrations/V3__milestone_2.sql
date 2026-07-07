CREATE INDEX idx_files_timeline_filters
    ON files (indexed_folder_id, is_present, extension, name);

CREATE INDEX idx_file_events_type_timeline
    ON file_events (event_type, detected_at DESC, id DESC);

CREATE INDEX idx_file_events_file_history
    ON file_events (file_id, detected_at DESC, id DESC);
