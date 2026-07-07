use std::path::{Path, PathBuf};

use crate::{
    database::Database,
    errors::ChronicleError,
    models::{
        FolderRegistration, IndexedFolder, NestedFolderWarning, NestingRelationship,
        RemoveIndexedFolderRequest,
    },
    platform::{
        NativePathNormalizer, PathNormalizer, contains_parent_component, is_strict_descendant,
        probe_directory,
    },
};

pub fn list_folders(database: &Database) -> Result<Vec<IndexedFolder>, ChronicleError> {
    let folders = database.list_indexed_folders()?;
    for folder in &folders {
        database.update_folder_availability(
            folder.id,
            probe_directory(Path::new(&folder.normalized_path)),
        )?;
    }
    database.list_indexed_folders()
}

pub fn register_folder(
    database: &Database,
    raw_path: &str,
) -> Result<FolderRegistration, ChronicleError> {
    let requested = PathBuf::from(raw_path);
    if contains_parent_component(&requested) {
        return Err(ChronicleError::Inaccessible);
    }
    let normalizer = NativePathNormalizer;
    let canonical = normalizer.canonical_directory(&requested)?;
    let normalized_path = normalizer.normalize(&canonical)?;
    let key = normalizer.comparison_key(&normalized_path);
    let existing = database.list_indexed_folders()?;
    let mut nested_warnings = Vec::new();
    for folder in &existing {
        let existing_path = Path::new(&folder.normalized_path);
        let relationship = if is_strict_descendant(&canonical, existing_path) {
            Some(NestingRelationship::InsideExisting)
        } else if is_strict_descendant(existing_path, &canonical) {
            Some(NestingRelationship::ContainsExisting)
        } else {
            None
        };
        if let Some(relationship) = relationship {
            nested_warnings.push(NestedFolderWarning {
                existing_folder_id: folder.id,
                existing_path: folder.normalized_path.clone(),
                relationship,
            });
        }
    }
    let display_name = canonical
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::to_owned)
        .unwrap_or_else(|| normalized_path.clone());
    let folder = database.insert_indexed_folder(&normalized_path, &key, &display_name)?;
    Ok(FolderRegistration {
        folder,
        nested_warnings,
    })
}

pub fn remove_folder(
    database: &Database,
    request: RemoveIndexedFolderRequest,
) -> Result<(), ChronicleError> {
    if !request.confirm_original_files_untouched {
        return Err(ChronicleError::ConfirmationRequired);
    }
    database.remove_indexed_folder(request.folder_id)
}

#[cfg(test)]
pub fn normalized_for_test(path: &Path) -> Result<String, ChronicleError> {
    crate::platform::path_to_string(path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use crate::{database::Database, errors::ChronicleError, models::RemoveIndexedFolderRequest};

    use super::{register_folder, remove_folder};

    fn database(directory: &TempDir) -> Database {
        Database::open(directory.path().join("chronicle.sqlite3"))
            .unwrap_or_else(|error| panic!("test database should open: {error}"))
    }

    #[test]
    fn registration_persists_and_exact_duplicates_are_rejected() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let db_path = directory.path().join("chronicle.sqlite3");
        let db = Database::open(&db_path)
            .unwrap_or_else(|error| panic!("test database should open: {error}"));
        let first = register_folder(&db, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"));
        assert!(!first.folder.monitoring_enabled);
        assert!(matches!(
            register_folder(&db, &root.to_string_lossy()),
            Err(ChronicleError::DuplicateFolder)
        ));
        drop(db);

        let reopened = Database::open(&db_path)
            .unwrap_or_else(|error| panic!("database should reopen: {error}"));
        let folders = reopened
            .list_indexed_folders()
            .unwrap_or_else(|error| panic!("folders should load: {error}"));
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].id, first.folder.id);
    }

    #[test]
    fn nested_roots_are_allowed_with_an_explicit_warning() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let outer = directory.path().join("outer");
        let inner = outer.join("inner");
        fs::create_dir_all(&inner)
            .unwrap_or_else(|error| panic!("nested root should exist: {error}"));
        let db = database(&directory);
        register_folder(&db, &outer.to_string_lossy())
            .unwrap_or_else(|error| panic!("outer should register: {error}"));
        let registration = register_folder(&db, &inner.to_string_lossy())
            .unwrap_or_else(|error| panic!("inner should register: {error}"));
        assert_eq!(registration.nested_warnings.len(), 1);
    }

    #[test]
    fn traversal_components_are_rejected_at_the_registration_boundary() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let attempted = root.join("..").join("root");
        let result = register_folder(&database(&directory), &attempted.to_string_lossy());
        assert!(matches!(result, Err(ChronicleError::Inaccessible)));
    }

    #[test]
    fn removing_an_index_never_changes_original_files() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        fs::create_dir(&root).unwrap_or_else(|error| panic!("root should exist: {error}"));
        let original = root.join("keep.txt");
        fs::write(&original, b"keep me")
            .unwrap_or_else(|error| panic!("original should be written: {error}"));
        let db = database(&directory);
        let folder = register_folder(&db, &root.to_string_lossy())
            .unwrap_or_else(|error| panic!("folder should register: {error}"))
            .folder;
        assert!(matches!(
            remove_folder(
                &db,
                RemoveIndexedFolderRequest {
                    folder_id: folder.id,
                    confirm_original_files_untouched: false,
                }
            ),
            Err(ChronicleError::ConfirmationRequired)
        ));
        remove_folder(
            &db,
            RemoveIndexedFolderRequest {
                folder_id: folder.id,
                confirm_original_files_untouched: true,
            },
        )
        .unwrap_or_else(|error| panic!("confirmed removal should succeed: {error}"));
        assert_eq!(
            fs::read(&original).unwrap_or_else(|error| panic!("original should remain: {error}")),
            b"keep me"
        );
        assert!(root.exists());
    }
}
