use crate::errors::ChronicleError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanRequest {
    pub indexed_folder_id: i64,
    pub normalized_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanSummary {
    pub files_seen: u64,
    pub warning_count: u64,
    pub error_count: u64,
}

pub trait MetadataScanner {
    fn scan_metadata(&self, request: &ScanRequest) -> Result<ScanSummary, ChronicleError>;
}
