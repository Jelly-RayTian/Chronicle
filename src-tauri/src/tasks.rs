use serde::{Deserialize, Serialize};

use crate::models::TaskStatus;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskSnapshot {
    pub id: String,
    pub status: TaskStatus,
    pub completed_units: u64,
    pub total_units: Option<u64>,
}

impl TaskSnapshot {
    #[must_use]
    pub fn idle(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: TaskStatus::Idle,
            completed_units: 0,
            total_units: None,
        }
    }
}
