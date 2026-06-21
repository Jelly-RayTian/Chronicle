use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed,
    Moved,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EventSource {
    Scan,
    Watcher,
    Reconciliation,
    Manual,
}
