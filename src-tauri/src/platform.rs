use std::{
    fs, io,
    path::{Component, Path, PathBuf},
    process::Command,
};

use crate::{errors::ChronicleError, models::AvailabilityStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformKind {
    Windows,
    MacOs,
    Linux,
    Other,
}

pub fn resolve_authorized_file(path: &Path, root: &Path) -> Result<PathBuf, ChronicleError> {
    let root_metadata = fs::symlink_metadata(root).map_err(|_| ChronicleError::FileUnavailable)?;
    if root_metadata.file_type().is_symlink() || !root_metadata.is_dir() {
        return Err(ChronicleError::FileUnavailable);
    }
    let metadata = fs::symlink_metadata(path).map_err(|_| ChronicleError::FileUnavailable)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ChronicleError::FileUnavailable);
    }
    let canonical_root = fs::canonicalize(root).map_err(|_| ChronicleError::FileUnavailable)?;
    let canonical_path = fs::canonicalize(path).map_err(|_| ChronicleError::FileUnavailable)?;
    if !canonical_path.starts_with(canonical_root) {
        return Err(ChronicleError::FileUnavailable);
    }
    Ok(canonical_path)
}

pub fn open_path(path: &Path) -> Result<(), ChronicleError> {
    let mut command = match PlatformKind::current() {
        PlatformKind::Windows => Command::new("explorer.exe"),
        PlatformKind::MacOs => Command::new("open"),
        PlatformKind::Linux => Command::new("xdg-open"),
        PlatformKind::Other => return Err(ChronicleError::Inaccessible),
    };
    command
        .arg(path)
        .spawn()
        .map(|_child| ())
        .map_err(Into::into)
}

pub fn reveal_path(path: &Path) -> Result<(), ChronicleError> {
    let mut command = match PlatformKind::current() {
        PlatformKind::Windows => {
            let mut value = Command::new("explorer.exe");
            value.arg(format!("/select,{}", path.to_string_lossy()));
            value
        }
        PlatformKind::MacOs => {
            let mut value = Command::new("open");
            value.arg("-R").arg(path);
            value
        }
        PlatformKind::Linux => {
            let mut value = Command::new("xdg-open");
            value.arg(path.parent().ok_or(ChronicleError::PathEncoding)?);
            value
        }
        PlatformKind::Other => return Err(ChronicleError::Inaccessible),
    };
    command.spawn().map(|_child| ()).map_err(Into::into)
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
    fn file_actions_require_a_regular_file_inside_the_authorized_root() {
        let directory = tempfile::tempdir()
            .unwrap_or_else(|error| panic!("temporary directory should exist: {error}"));
        let root = directory.path().join("root");
        std::fs::create_dir(&root).unwrap_or_else(|error| panic!("{error}"));
        let inside = root.join("inside.txt");
        let outside = directory.path().join("outside.txt");
        std::fs::write(&inside, b"inside").unwrap_or_else(|error| panic!("{error}"));
        std::fs::write(&outside, b"outside").unwrap_or_else(|error| panic!("{error}"));

        assert!(super::resolve_authorized_file(&inside, &root).is_ok());
        assert!(super::resolve_authorized_file(&outside, &root).is_err());
        assert!(super::resolve_authorized_file(&root.join("missing.txt"), &root).is_err());
    }

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
