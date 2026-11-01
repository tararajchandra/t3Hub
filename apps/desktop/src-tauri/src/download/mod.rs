pub mod manager;

use serde::{Deserialize, Serialize};

pub use manager::{DownloadError, DownloadManager, ProgressUpdate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Queued,
    Downloading {
        bytes_downloaded: u64,
        total_bytes: Option<u64>,
        percentage: f32,
    },
    Verifying,
    Completed {
        file_path: String,
    },
    Failed {
        error: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub artifact_id: String,
    pub url: String,
    pub expected_sha256: String,
    pub status: DownloadStatus,
}
