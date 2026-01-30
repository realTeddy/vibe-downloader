//! Database schema types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Download status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Queued => "queued",
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
    
    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "queued" => Self::Queued,
            "downloading" => Self::Downloading,
            "paused" => Self::Paused,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            _ => Self::Pending,
        }
    }
}

/// A download record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub file_type: String,
    pub destination: PathBuf,
    pub total_size: Option<u64>,
    pub downloaded_size: u64,
    pub status: DownloadStatus,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl DownloadRecord {
    /// Create a new download record
    pub fn new(
        url: String,
        filename: String,
        file_type: String,
        destination: PathBuf,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            url,
            filename,
            file_type,
            destination,
            total_size: None,
            downloaded_size: 0,
            status: DownloadStatus::Pending,
            error_message: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }
    
    /// Get progress as a percentage (0.0 - 100.0)
    pub fn progress(&self) -> f64 {
        match self.total_size {
            Some(total) if total > 0 => (self.downloaded_size as f64 / total as f64) * 100.0,
            _ => 0.0,
        }
    }
}
