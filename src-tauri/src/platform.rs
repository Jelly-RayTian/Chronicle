use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

use crate::{errors::ChronicleError, models::AvailabilityStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind {
    Windows,
    MacOs,
    Linux,
    Other,
}

impl PlatformKind {
    #[must_use]
    pub fn current() -> Self {
        match std::env::consts::OS {
            "windows" => Self::Windows,
            "macos" => Self::MacOs,
            "linux" => Self::Linux,
            _ => Self::Other,
        }
    }
}

pub trait PathNormalizer {
    fn normalize(&self, path: &Path) -> Result<String, ChronicleError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NativePathNormalizer;

impl NativePathNormalizer {
    pub fn canonical_directory(&self, path: &Path) -> Result<PathBuf, ChronicleError> {
        let metadata = fs::symlink_metadata(path).map_err(classify_io_error)?;
        if metadata.file_type().is_symlink() {
            return Err(ChronicleError::SymbolicLinkRoot);
        }
        if !metadata.is_dir() {
            return Err(ChronicleError::NotDirectory);
        }
        let canonical = fs::canonicalize(path).map_err(classify_io_error)?;
        Ok(PathBuf::from(path_to_string(&canonical)?))
    }

    pub fn comparison_key(&self, normalized_path: &str) -> String {
        if matches!(PlatformKind::current(), PlatformKind::Windows) {
            normalized_path.to_lowercase()
        } else {
            normalized_path.to_owned()
        }
    }
}

impl PathNormalizer for NativePathNormalizer {
    fn normalize(&self, path: &Path) -> Result<String, ChronicleError> {
        let canonical = fs::canonicalize(path).map_err(classify_io_error)?;
        path_to_string(&canonical)
    }
}

pub fn path_to_string(path: &Path) -> Result<String, ChronicleError> {
    let value = path.to_str().ok_or(ChronicleError::PathEncoding)?;
    #[cfg(windows)]
    let value = value.strip_prefix(r"\\?\").unwrap_or(value);
    Ok(value.to_owned())
}

pub fn classify_io_error(error: io::Error) -> ChronicleError {
    match error.kind() {
        io::ErrorKind::NotFound => ChronicleError::MissingOrMoved,
        io::ErrorKind::PermissionDenied => ChronicleError::PermissionDenied,
        _ => ChronicleError::Inaccessible,
    }
}

pub fn probe_directory(path: &Path) -> AvailabilityStatus {
    match fs::read_dir(path) {
        Ok(_) => AvailabilityStatus::Available,
        Err(error) => match error.kind() {
            io::ErrorKind::NotFound => AvailabilityStatus::MissingOrMoved,
            io::ErrorKind::PermissionDenied => AvailabilityStatus::PermissionDenied,
            _ => AvailabilityStatus::Inaccessible,
        },
    }
}

pub fn is_strict_descendant(path: &Path, possible_parent: &Path) -> bool {
    path != possible_parent && path.starts_with(possible_parent)
}

pub fn contains_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use std::io;

    use crate::{errors::ChronicleError, models::AvailabilityStatus};

    use super::{classify_io_error, probe_directory};

    #[test]
    fn permission_and_missing_errors_remain_distinct() {
        assert!(matches!(
            classify_io_error(io::Error::from(io::ErrorKind::PermissionDenied)),
            ChronicleError::PermissionDenied
        ));
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        assert_eq!(
            probe_directory(&directory.path().join("missing")),
            AvailabilityStatus::MissingOrMoved
        );
    }
}
