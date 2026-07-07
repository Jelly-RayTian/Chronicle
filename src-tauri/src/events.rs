use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed,
    Moved,
    LikelyRenamed,
    PossibleMove,
}

impl FileEventType {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
            Self::Renamed => "renamed",
            Self::Moved => "moved",
            Self::LikelyRenamed => "likely_renamed",
            Self::PossibleMove => "possible_move",
        }
    }

    #[must_use]
    pub fn is_rename_like(self) -> bool {
        matches!(
            self,
            Self::Renamed | Self::Moved | Self::LikelyRenamed | Self::PossibleMove
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EventSource {
    Scan,
    Watcher,
    Reconciliation,
    Manual,
}
