ALTER TABLE indexed_folders
    ADD COLUMN content_indexing_enabled INTEGER NOT NULL DEFAULT 0;

ALTER TABLE indexed_folders
    ADD COLUMN content_indexing_extensions TEXT NOT NULL DEFAULT 'txt,md,rs,js,ts,jsx,tsx,py,go,java,c,cpp,h,hpp,swift,kotlin,rb,php,json,yaml,yml,toml,sh,bash,zsh,ps1,html,css,scss,sql';

ALTER TABLE indexed_folders
    ADD COLUMN content_indexing_max_bytes INTEGER NOT NULL DEFAULT 1048576;

ALTER TABLE indexed_folders
    ADD COLUMN content_indexing_exclusion_patterns TEXT NOT NULL DEFAULT '.env,.env.*,*.key,*.pem,*.crt,*.p12,*.pfx,id_rsa,id_ed25519,id_ecdsa,.htpasswd,.npmrc,.pypirc,netrc';

CREATE TABLE content_index_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL UNIQUE REFERENCES files(id) ON DELETE CASCADE,
    indexed_at TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    word_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_content_index_documents_file_id ON content_index_documents(file_id);

CREATE VIRTUAL TABLE content_index_fts USING fts5(
    content,
    doc_id UNINDEXED
);
