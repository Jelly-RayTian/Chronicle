use std::{fs, path::Path};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileIdentity {
    Stable { key: String },
    Fingerprint { key: String },
}

impl FileIdentity {
    #[must_use]
    pub fn as_key(&self) -> Option<&str> {
        match self {
            Self::Stable { key } | Self::Fingerprint { key } => {
                if key.is_empty() {
                    None
                } else {
                    Some(key.as_str())
                }
            }
        }
    }

    #[must_use]
    pub fn is_stable(&self) -> bool {
        matches!(self, Self::Stable { .. })
    }
}

pub fn resolve_file_identity(path: &Path) -> Result<FileIdentity, std::io::Error> {
    let metadata = fs::symlink_metadata(path)?;

    let stable = try_stable_identity(&metadata);
    if let Some(identity) = stable {
        return Ok(identity);
    }

    let fp = build_fingerprint(&metadata);
    Ok(FileIdentity::Fingerprint { key: fp })
}

#[cfg(unix)]
fn try_stable_identity(metadata: &fs::Metadata) -> Option<FileIdentity> {
    use std::os::unix::fs::MetadataExt;
    let dev = metadata.dev();
    let ino = metadata.ino();
    if dev != 0 || ino != 0 {
        Some(FileIdentity::Stable {
            key: format!("ino:{dev}:{ino}"),
        })
    } else {
        None
    }
}

#[cfg(not(unix))]
fn try_stable_identity(_metadata: &fs::Metadata) -> Option<FileIdentity> {
    None
}

fn build_fingerprint(metadata: &fs::Metadata) -> String {
    let size = metadata.len();
    let modified = metadata
        .modified()
        .ok()
        .map(|time| -> u64 {
            time.duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0)
        })
        .unwrap_or(0);

    let created = metadata
        .created()
        .ok()
        .map(|time| -> u64 {
            time.duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0)
        })
        .unwrap_or(0);

    format!("fp:{}:{}:{}", size, modified, created)
}

pub fn identity_key_match(a: Option<&str>, b: Option<&str>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) if !a.is_empty() && !b.is_empty() => a == b,
        _ => false,
    }
}
