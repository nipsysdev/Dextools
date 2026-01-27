use serde::{Deserialize, Serialize};
use storage_bindings::DebugInfo;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StorageConnectionStatus {
    Disconnected,
    Initialized,
    Connecting,
    Connected,
    Error,
}

impl Default for StorageConnectionStatus {
    fn default() -> Self {
        StorageConnectionStatus::Disconnected
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub peer_id: Option<String>,
    pub version: Option<String>,
    pub repo_path: Option<String>,
    pub debug_info: Option<DebugInfo>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadResultResponse {
    pub cid: String,
    pub size: usize,
    pub duration_ms: u64,
    pub verified: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DownloadResultResponse {
    pub cid: String,
    pub size: usize,
    pub duration_ms: u64,
    pub verified: bool,
    pub filepath: Option<String>,
}
