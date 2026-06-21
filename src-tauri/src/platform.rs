use std::path::Path;

use crate::errors::ChronicleError;

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
