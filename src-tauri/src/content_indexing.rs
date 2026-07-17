use std::{
    collections::HashSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{
    database::Database,
    errors::ChronicleError,
    models::{
        ContentSearchMode, ContentSearchResult, FileRecord, IndexedFolder,
        ReindexFolderContentResponse, SearchFilesRequest,
    },
    platform::classify_io_error,
    watcher::WatcherObservation,
};

const DEFAULT_MAX_BYTES: i64 = 1_048_576;
const DEFAULT_EXTENSIONS: &str = "txt,md,rs,js,ts,jsx,tsx,py,go,java,c,cpp,h,hpp,swift,kotlin,rb,php,json,yaml,yml,toml,sh,bash,zsh,ps1,html,css,scss,sql";
const DEFAULT_EXCLUSION_PATTERNS: &str = ".env,.env.*,*.key,*.pem,*.crt,*.p12,*.pfx,id_rsa,id_ed25519,id_ecdsa,.htpasswd,.npmrc,.pypirc,netrc";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolderContentConfig {
    pub enabled: bool,
    pub max_bytes: i64,
}

impl Default for FolderContentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_bytes: DEFAULT_MAX_BYTES,
        }
    }
}

#[must_use]
pub fn default_extensions() -> String {
    DEFAULT_EXTENSIONS.to_owned()
}

#[must_use]
pub fn default_exclusion_patterns() -> String {
    DEFAULT_EXCLUSION_PATTERNS.to_owned()
}

#[must_use]
pub fn default_max_bytes() -> i64 {
    DEFAULT_MAX_BYTES
}

fn parse_extensions(value: &str) -> HashSet<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_start_matches('.').to_lowercase())
        .collect()
}

fn parse_patterns(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn glob_matches(pattern: &str, text: &str) -> bool {
    let pattern = pattern.to_lowercase();
    let text = text.to_lowercase();
    if !pattern.contains('*') {
        return pattern == text;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let first = parts.first().copied().unwrap_or("");
    let last = parts.last().copied().unwrap_or("");
    if !first.is_empty() && !text.starts_with(first) {
        return false;
    }
    if !last.is_empty() && !text.ends_with(last) {
        return false;
    }
    let end_bound = text
        .len()
        .saturating_sub(if last.is_empty() { 0 } else { last.len() });
    let mut position = first.len();
    let middle_count = parts.len().saturating_sub(2);
    for part in parts.iter().skip(1).take(middle_count) {
        if part.is_empty() {
            continue;
        }
        if position > end_bound {
            return false;
        }
        match text[position..end_bound].find(part) {
            Some(index) => position += index + part.len(),
            None => return false,
        }
    }
    true
}

fn is_excluded(name: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| glob_matches(pattern, name))
}

fn is_eligible(file: &FileRecord, folder: &IndexedFolder) -> bool {
    if !file.is_present {
        return false;
    }
    if file.size_bytes > folder.content_indexing_max_bytes {
        return false;
    }
    let extensions = parse_extensions(&folder.content_indexing_extensions);
    let extension = file.extension.as_deref().unwrap_or("").to_lowercase();
    if !extensions.contains(&extension) {
        return false;
    }
    let patterns = parse_patterns(&folder.content_indexing_exclusion_patterns);
    !is_excluded(&file.name, &patterns)
}

fn read_limited_text(path: &Path, max_bytes: i64) -> Result<Option<String>, ChronicleError> {
    let max_bytes_usize = usize::try_from(max_bytes.max(0)).unwrap_or(usize::MAX);
    let metadata = fs::symlink_metadata(path).map_err(classify_io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Ok(None);
    }
    if metadata.len() > u64::try_from(max_bytes).unwrap_or(u64::MAX) {
        return Ok(None);
    }
    let mut file = fs::File::open(path).map_err(classify_io_error)?;
    let mut buffer = vec![0; max_bytes_usize.saturating_add(1)];
    let read = file.read(&mut buffer).map_err(classify_io_error)?;
    if read > max_bytes_usize {
        return Ok(None);
    }
    buffer.truncate(read);
    Ok(Some(String::from_utf8_lossy(&buffer).into_owned()))
}

fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn sanitize_snippet(snippet: &str) -> String {
    let cleaned = snippet.replace("<<<", "").replace(">>>", "");
    cleaned
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn reindex_file(
    database: &Database,
    file: &FileRecord,
    folder: &IndexedFolder,
) -> Result<bool, ChronicleError> {
    if !folder.content_indexing_enabled || !is_eligible(file, folder) {
        database.delete_content_index_for_file(file.id)?;
        return Ok(false);
    }
    let path = PathBuf::from(&file.normalized_path);
    let text = match read_limited_text(&path, folder.content_indexing_max_bytes) {
        Ok(Some(text)) => text,
        Ok(None) => {
            database.delete_content_index_for_file(file.id)?;
            return Ok(false);
        }
        Err(error) => {
            database.delete_content_index_for_file(file.id)?;
            return Err(error);
        }
    };
    let words = word_count(&text);
    database.content_index_document(file.id, &text, file.size_bytes, words)?;
    Ok(true)
}

pub fn sync_folder(
    database: &Database,
    folder_id: i64,
) -> Result<ReindexFolderContentResponse, ChronicleError> {
    let folder = database.get_indexed_folder(folder_id)?;
    if !folder.content_indexing_enabled {
        database.delete_content_index_for_folder(folder_id)?;
        return Ok(ReindexFolderContentResponse {
            files_indexed: 0,
            files_removed: 0,
        });
    }

    let files = database.list_present_files_for_content_indexing(folder_id)?;
    let indexed = database.content_indexed_documents_for_folder(folder_id)?;
    let indexed_map: std::collections::HashMap<i64, (i64, String, String)> = indexed
        .into_iter()
        .map(|(file_id, size, indexed_at, modified_at)| (file_id, (size, indexed_at, modified_at)))
        .collect();

    let mut files_indexed = 0_usize;
    let mut files_removed = 0_usize;

    for file in &files {
        let needs_reindex = match indexed_map.get(&file.id) {
            None => true,
            Some((size, _indexed_at, modified_at)) => {
                *size != file.size_bytes || modified_at != &file.filesystem_modified_at
            }
        };
        if needs_reindex && reindex_file(database, file, &folder)? {
            files_indexed += 1;
        }
    }

    let present_ids: HashSet<i64> = files.iter().map(|f| f.id).collect();
    for file_id in indexed_map.keys() {
        if !present_ids.contains(file_id) {
            database.delete_content_index_for_file(*file_id)?;
            files_removed += 1;
        }
    }

    Ok(ReindexFolderContentResponse {
        files_indexed,
        files_removed,
    })
}

pub fn sync_files_from_observations(
    database: &Database,
    folder_id: i64,
    observations: &[WatcherObservation],
) -> Result<(), ChronicleError> {
    let folder = match database.get_indexed_folder(folder_id) {
        Ok(folder) => folder,
        Err(_) => return Ok(()),
    };
    if !folder.content_indexing_enabled {
        return Ok(());
    }
    for observation in observations {
        match observation {
            WatcherObservation::Present(file) => {
                let record = database
                    .list_present_files_for_content_indexing(folder_id)?
                    .into_iter()
                    .find(|f| f.normalized_path == file.normalized_path);
                if let Some(record) = record {
                    let _ = reindex_file(database, &record, &folder);
                }
            }
            WatcherObservation::Missing { normalized_path } => {
                if let Some(file_id) = database
                    .list_present_files_for_content_indexing(folder_id)?
                    .into_iter()
                    .find(|f| f.normalized_path == *normalized_path)
                    .map(|f| f.id)
                {
                    let _ = database.delete_content_index_for_file(file_id);
                }
            }
        }
    }
    Ok(())
}

pub fn remove_folder_content_index(
    database: &Database,
    folder_id: i64,
) -> Result<(), ChronicleError> {
    database.delete_content_index_for_folder(folder_id)
}

pub fn clear_all_content_index(database: &Database) -> Result<(), ChronicleError> {
    database.clear_content_index()
}

pub fn search_files(
    database: &Database,
    request: SearchFilesRequest,
) -> Result<Vec<ContentSearchResult>, ChronicleError> {
    let query = request.query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let limit = request.limit.clamp(1, 200);
    let ext = request
        .extension
        .as_deref()
        .filter(|e| !e.trim().is_empty());
    let from = request.date_from.as_deref();
    let to = request.date_to.as_deref();
    let presence_bool = request
        .presence
        .map(|p| matches!(p, crate::models::PresenceFilter::Present));
    match request.mode {
        ContentSearchMode::Filename => database
            .search_filename(
                query,
                request.folder_id,
                ext,
                from,
                to,
                presence_bool,
                limit,
            )
            .map(|files| {
                files
                    .into_iter()
                    .map(|file| ContentSearchResult {
                        file,
                        snippet: String::new(),
                        rank: 0.0,
                    })
                    .collect()
            }),
        ContentSearchMode::Content => database
            .search_content(query, request.folder_id, ext, from, to, limit)
            .map(|results| {
                results
                    .into_iter()
                    .map(|result| ContentSearchResult {
                        file: result.file,
                        snippet: sanitize_snippet(&result.snippet),
                        rank: result.rank,
                    })
                    .collect()
            }),
    }
}

pub fn read_file_text_for_test(
    path: &Path,
    max_bytes: i64,
) -> Result<Option<String>, ChronicleError> {
    read_limited_text(path, max_bytes)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{
        clear_all_content_index, glob_matches, is_eligible, is_excluded, parse_extensions,
        parse_patterns, remove_folder_content_index, sanitize_snippet, search_files, sync_folder,
    };
    use crate::models::FileRecord;
    use crate::models::{ContentSearchMode, SearchFilesRequest};

    fn file(name: &str, extension: Option<&str>, size: i64, present: bool) -> FileRecord {
        FileRecord {
            id: 1,
            indexed_folder_id: 1,
            normalized_path: PathBuf::from("/root")
                .join(name)
                .to_string_lossy()
                .into_owned(),
            name: name.to_owned(),
            parent_path: "/root".to_owned(),
            extension: extension.map(str::to_owned),
            size_bytes: size,
            filesystem_created_at: None,
            filesystem_modified_at: "2026-01-01T00:00:00.000Z".to_owned(),
            first_indexed_at: "2026-01-01T00:00:00.000Z".to_owned(),
            last_seen_at: "2026-01-01T00:00:00.000Z".to_owned(),
            is_present: present,
        }
    }

    fn folder() -> crate::models::IndexedFolder {
        crate::models::IndexedFolder {
            id: 1,
            normalized_path: "/root".to_owned(),
            display_name: "root".to_owned(),
            added_at: "2026-01-01T00:00:00.000Z".to_owned(),
            last_successful_scan_at: None,
            monitoring_enabled: false,
            availability_status: crate::models::AvailabilityStatus::Available,
            last_checked_at: None,
            content_indexing_enabled: true,
            content_indexing_extensions: "txt,md,rs".to_owned(),
            content_indexing_max_bytes: 1024,
            content_indexing_exclusion_patterns: ".env,.env.*,*.key".to_owned(),
        }
    }

    #[test]
    fn supported_extensions_are_parsed_case_insensitively() {
        let extensions = parse_extensions("TXT, Md , rs");
        assert!(extensions.contains("txt"));
        assert!(extensions.contains("md"));
        assert!(extensions.contains("rs"));
    }

    #[test]
    fn secret_and_wildcard_exclusions_match_expected_files() {
        let patterns = parse_patterns(".env,.env.*,*.key,id_rsa");
        assert!(is_excluded(".env", &patterns));
        assert!(is_excluded(".env.local", &patterns));
        assert!(is_excluded("secret.key", &patterns));
        assert!(is_excluded("id_rsa", &patterns));
        assert!(!is_excluded("readme.txt", &patterns));
    }

    #[test]
    fn glob_pattern_handles_leading_and_trailing_stars() {
        assert!(glob_matches("*.key", "secret.key"));
        assert!(!glob_matches("*.key", "secret.pem"));
        assert!(glob_matches(".env.*", ".env.local"));
        assert!(glob_matches("id_rsa", "id_rsa"));
        assert!(!glob_matches("id_rsa", "id_rsa.pub"));
    }

    #[test]
    fn eligibility_respects_presence_extension_size_and_exclusion() {
        let folder = folder();
        assert!(is_eligible(
            &file("note.txt", Some("txt"), 100, true),
            &folder
        ));
        assert!(!is_eligible(
            &file("note.txt", Some("txt"), 100, false),
            &folder
        ));
        assert!(!is_eligible(
            &file("note.bin", Some("bin"), 100, true),
            &folder
        ));
        assert!(!is_eligible(
            &file("note.txt", Some("txt"), 2048, true),
            &folder
        ));
        assert!(!is_eligible(&file(".env", Some(""), 10, true), &folder));
        assert!(!is_eligible(
            &file("secret.key", Some("key"), 10, true),
            &folder
        ));
    }

    #[test]
    fn snippet_markers_are_stripped_and_html_escaped() {
        let raw = "<<<hello>>> <script>alert('x')</script>";
        let sanitized = sanitize_snippet(raw);
        assert!(!sanitized.contains("<<<"));
        assert!(!sanitized.contains(">>>"));
        assert!(sanitized.contains("&lt;script&gt;"));
        assert!(sanitized.contains("&#x27;"));
    }

    fn scan_folder(database: &crate::database::Database, root: &std::path::Path, folder_id: i64) {
        let run = database
            .create_scan_run(folder_id)
            .unwrap_or_else(|error| panic!("scan should start: {error}"));
        let counts = crate::scanner::traverse(
            database,
            run.scan_run_id,
            folder_id,
            root,
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap_or_else(|error| panic!("scan should traverse: {error}"));
        database
            .complete_scan(
                run.scan_run_id,
                folder_id,
                counts.files_seen,
                counts.warnings,
                counts.errors,
            )
            .unwrap_or_else(|error| panic!("scan should publish: {error}"));
    }

    #[test]
    fn content_indexing_is_disabled_by_default() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        fs::write(root.join("note.txt"), b"hello world")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        scan_folder(&database, &root, folder.id);
        let result = sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        assert_eq!(result.files_indexed, 0);
    }

    #[test]
    fn txt_and_markdown_files_are_indexed_and_searchable() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        fs::write(root.join("note.txt"), b"hello world")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        fs::write(root.join("doc.md"), b"# Chronicle\nlocal search")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        database
            .set_folder_content_indexing(folder.id, true, None, None, None)
            .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        scan_folder(&database, &root, folder.id);
        let result = sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        assert_eq!(result.files_indexed, 2);
        let hits = search_files(
            &database,
            SearchFilesRequest {
                query: "world".to_owned(),
                mode: ContentSearchMode::Content,
                folder_id: Some(folder.id),
                limit: 10,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].file.name, "note.txt");
    }

    #[test]
    fn secret_files_are_excluded_from_content_index() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        fs::write(root.join(".env"), b"SECRET_KEY=abc123")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        fs::write(root.join("readme.txt"), b"public content")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        database
            .set_folder_content_indexing(folder.id, true, None, None, None)
            .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        scan_folder(&database, &root, folder.id);
        let result = sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        assert_eq!(result.files_indexed, 1);
        let hits = search_files(
            &database,
            SearchFilesRequest {
                query: "SECRET_KEY".to_owned(),
                mode: ContentSearchMode::Content,
                folder_id: Some(folder.id),
                limit: 10,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert!(hits.is_empty());
    }

    #[test]
    fn oversized_files_are_excluded_from_content_index() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        fs::write(root.join("small.txt"), b"tiny")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        fs::write(root.join("big.txt"), vec![b'a'; 100])
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        database
            .set_folder_content_indexing(folder.id, true, None, Some(50), None)
            .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        scan_folder(&database, &root, folder.id);
        let result = sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        assert_eq!(result.files_indexed, 1);
        let hits = search_files(
            &database,
            SearchFilesRequest {
                query: "aaaa".to_owned(),
                mode: ContentSearchMode::Content,
                folder_id: Some(folder.id),
                limit: 10,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert!(hits.is_empty());
    }

    #[test]
    fn incremental_update_replaces_stale_content() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let path = root.join("draft.txt");
        fs::write(&path, b"first version")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        database
            .set_folder_content_indexing(folder.id, true, None, None, None)
            .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        scan_folder(&database, &root, folder.id);
        sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        assert_eq!(
            search_files(
                &database,
                SearchFilesRequest {
                    query: "first".to_owned(),
                    mode: ContentSearchMode::Content,
                    folder_id: Some(folder.id),
                    limit: 10,
                    extension: None,
                    date_from: None,
                    date_to: None,
                    presence: None,
                    event_type: None,
                },
            )
            .unwrap_or_else(|error| panic!("search should succeed: {error}"))
            .len(),
            1
        );
        fs::write(&path, b"second version")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        scan_folder(&database, &root, folder.id);
        sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        assert_eq!(
            search_files(
                &database,
                SearchFilesRequest {
                    query: "second".to_owned(),
                    mode: ContentSearchMode::Content,
                    folder_id: Some(folder.id),
                    limit: 10,
                    extension: None,
                    date_from: None,
                    date_to: None,
                    presence: None,
                    event_type: None,
                },
            )
            .unwrap_or_else(|error| panic!("search should succeed: {error}"))
            .len(),
            1
        );
        assert!(
            search_files(
                &database,
                SearchFilesRequest {
                    query: "first".to_owned(),
                    mode: ContentSearchMode::Content,
                    folder_id: Some(folder.id),
                    limit: 10,
                    extension: None,
                    date_from: None,
                    date_to: None,
                    presence: None,
                    event_type: None,
                },
            )
            .unwrap_or_else(|error| panic!("search should succeed: {error}"))
            .is_empty(),
            "old content should no longer match"
        );
    }

    #[test]
    fn deleting_a_file_removes_its_content_index() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let path = root.join("temp.txt");
        fs::write(&path, b"find me")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        database
            .set_folder_content_indexing(folder.id, true, None, None, None)
            .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        scan_folder(&database, &root, folder.id);
        sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        fs::remove_file(&path).unwrap_or_else(|error| panic!("file should be removed: {error}"));
        scan_folder(&database, &root, folder.id);
        sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        let hits = search_files(
            &database,
            SearchFilesRequest {
                query: "find".to_owned(),
                mode: ContentSearchMode::Content,
                folder_id: Some(folder.id),
                limit: 10,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert!(hits.is_empty());
    }

    #[test]
    fn disabling_content_indexing_removes_folder_index() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        fs::write(root.join("note.txt"), b"chronicle content")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        database
            .set_folder_content_indexing(folder.id, true, None, None, None)
            .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        scan_folder(&database, &root, folder.id);
        sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        database
            .set_folder_content_indexing(folder.id, false, None, None, None)
            .unwrap_or_else(|error| panic!("disable should succeed: {error}"));
        remove_folder_content_index(&database, folder.id)
            .unwrap_or_else(|error| panic!("remove should succeed: {error}"));
        let hits = search_files(
            &database,
            SearchFilesRequest {
                query: "chronicle".to_owned(),
                mode: ContentSearchMode::Content,
                folder_id: Some(folder.id),
                limit: 10,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert!(hits.is_empty());
    }

    #[test]
    fn clearing_content_index_does_not_delete_original_files() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let note = root.join("note.txt");
        fs::write(&note, b"persist on disk")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        database
            .set_folder_content_indexing(folder.id, true, None, None, None)
            .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        scan_folder(&database, &root, folder.id);
        sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        clear_all_content_index(&database)
            .unwrap_or_else(|error| panic!("clear should succeed: {error}"));
        assert!(
            note.exists(),
            "clearing the content index must not delete original files"
        );
        let hits = search_files(
            &database,
            SearchFilesRequest {
                query: "persist".to_owned(),
                mode: ContentSearchMode::Content,
                folder_id: Some(folder.id),
                limit: 10,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert!(hits.is_empty());
    }

    #[test]
    fn clear_all_content_index_removes_everything() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        fs::write(root.join("note.txt"), b"keep local")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        database
            .set_folder_content_indexing(folder.id, true, None, None, None)
            .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        scan_folder(&database, &root, folder.id);
        sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        clear_all_content_index(&database)
            .unwrap_or_else(|error| panic!("clear should succeed: {error}"));
        let hits = search_files(
            &database,
            SearchFilesRequest {
                query: "local".to_owned(),
                mode: ContentSearchMode::Content,
                folder_id: None,
                limit: 10,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert!(hits.is_empty());
    }

    #[test]
    fn filename_search_does_not_require_content_indexing() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        fs::write(root.join("budget.txt"), b"numbers")
            .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        scan_folder(&database, &root, folder.id);
        let hits = search_files(
            &database,
            SearchFilesRequest {
                query: "budget".to_owned(),
                mode: ContentSearchMode::Filename,
                folder_id: Some(folder.id),
                limit: 10,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].file.name, "budget.txt");
    }

    #[test]
    fn content_search_snippets_are_sanitized() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        fs::write(
            root.join("page.html"),
            b"<div>hello world</div> more text here for the snippet",
        )
        .unwrap_or_else(|error| panic!("file should be written: {error}"));
        let database = crate::database::Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("database should open: {error}"));
        let folder = crate::folders::register_folder(&database, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        database
            .set_folder_content_indexing(folder.id, true, Some("txt,md,html"), None, None)
            .unwrap_or_else(|error| panic!("enable should succeed: {error}"));
        scan_folder(&database, &root, folder.id);
        sync_folder(&database, folder.id)
            .unwrap_or_else(|error| panic!("sync should succeed: {error}"));
        let hits = search_files(
            &database,
            SearchFilesRequest {
                query: "world".to_owned(),
                mode: ContentSearchMode::Content,
                folder_id: Some(folder.id),
                limit: 10,
                extension: None,
                date_from: None,
                date_to: None,
                presence: None,
                event_type: None,
            },
        )
        .unwrap_or_else(|error| panic!("search should succeed: {error}"));
        assert_eq!(hits.len(), 1);
        let snippet = &hits[0].snippet;
        assert!(
            !snippet.contains('<'),
            "snippet should not contain raw angle brackets: {snippet}"
        );
        assert!(
            snippet.contains("&lt;div&gt;"),
            "snippet should escape html tags: {snippet}"
        );
    }
}
