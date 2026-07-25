UPDATE indexed_folders
SET content_indexing_max_bytes = 1
WHERE content_indexing_max_bytes < 1;

UPDATE indexed_folders
SET content_indexing_max_bytes = 8388608
WHERE content_indexing_max_bytes > 8388608;

CREATE TRIGGER validate_content_indexing_max_bytes_insert
BEFORE INSERT ON indexed_folders
WHEN NEW.content_indexing_max_bytes < 1 OR NEW.content_indexing_max_bytes > 8388608
BEGIN
    SELECT RAISE(ABORT, 'content_indexing_max_bytes must be between 1 and 8388608');
END;

CREATE TRIGGER validate_content_indexing_max_bytes_update
BEFORE UPDATE OF content_indexing_max_bytes ON indexed_folders
WHEN NEW.content_indexing_max_bytes < 1 OR NEW.content_indexing_max_bytes > 8388608
BEGIN
    SELECT RAISE(ABORT, 'content_indexing_max_bytes must be between 1 and 8388608');
END;
