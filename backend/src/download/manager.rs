//! Download manager for handling concurrent downloads with queue

use crate::db::{DownloadRecord, DownloadStatus};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Semaphore};

/// Progress update sent to clients
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgressUpdate {
    pub id: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub speed: u64, // bytes per second
    pub status: DownloadStatus,
    pub error: Option<String>,
}

/// Inner state that cannot be cloned directly
struct DownloadManagerInner {
    /// Maximum concurrent downloads
    max_concurrent: RwLock<usize>,
    
    /// Active downloads (id -> cancel sender)
    active: RwLock<HashMap<String, mpsc::Sender<()>>>,
    
    /// Queued downloads waiting to start
    queue: RwLock<VecDeque<DownloadRecord>>,
}

/// Download manager that handles concurrent downloads and queuing
#[derive(Clone)]
pub struct DownloadManager {
    /// Semaphore to limit concurrent downloads
    semaphore: Arc<Semaphore>,
    
    /// Inner state wrapped in Arc
    inner: Arc<DownloadManagerInner>,
    
    /// Broadcast channel for progress updates
    progress_tx: broadcast::Sender<ProgressUpdate>,
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(max_concurrent: usize) -> Self {
        let (progress_tx, _) = broadcast::channel(1000);
        
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            inner: Arc::new(DownloadManagerInner {
                max_concurrent: RwLock::new(max_concurrent),
                active: RwLock::new(HashMap::new()),
                queue: RwLock::new(VecDeque::new()),
            }),
            progress_tx,
        }
    }
    
    /// Subscribe to progress updates
    pub fn subscribe(&self) -> broadcast::Receiver<ProgressUpdate> {
        self.progress_tx.subscribe()
    }
    
    /// Get progress sender for tasks
    pub fn progress_sender(&self) -> broadcast::Sender<ProgressUpdate> {
        self.progress_tx.clone()
    }
    
    /// Get the semaphore for limiting concurrency
    pub fn semaphore(&self) -> Arc<Semaphore> {
        Arc::clone(&self.semaphore)
    }
    
    /// Add a download to the active set
    pub fn add_active(&self, id: String, cancel_tx: mpsc::Sender<()>) {
        self.inner.active.write().insert(id, cancel_tx);
    }
    
    /// Remove a download from the active set
    pub fn remove_active(&self, id: &str) {
        self.inner.active.write().remove(id);
    }
    
    /// Check if a download is active
    pub fn is_active(&self, id: &str) -> bool {
        self.inner.active.read().contains_key(id)
    }
    
    /// Cancel a download
    pub async fn cancel(&self, id: &str) -> bool {
        // Clone the sender if found to avoid holding the lock across await
        let cancel_tx = self.inner.active.read().get(id).cloned();
        
        if let Some(tx) = cancel_tx {
            let _ = tx.send(()).await;
            true
        } else {
            // Check if it's in the queue
            let mut queue = self.inner.queue.write();
            if let Some(pos) = queue.iter().position(|d| d.id == id) {
                queue.remove(pos);
                return true;
            }
            false
        }
    }
    
    /// Add a download to the queue
    pub fn enqueue(&self, download: DownloadRecord) {
        self.inner.queue.write().push_back(download);
    }
    
    /// Get next download from queue
    pub fn dequeue(&self) -> Option<DownloadRecord> {
        self.inner.queue.write().pop_front()
    }
    
    /// Get queue length
    pub fn queue_len(&self) -> usize {
        self.inner.queue.read().len()
    }
    
    /// Get active download count
    pub fn active_count(&self) -> usize {
        self.inner.active.read().len()
    }
    
    /// Update max concurrent downloads
    pub fn set_max_concurrent(&self, max: usize) {
        let mut current_max = self.inner.max_concurrent.write();
        let old_max = *current_max;
        *current_max = max;
        
        // If increasing, add permits
        if max > old_max {
            self.semaphore.add_permits(max - old_max);
        }
        // Note: Decreasing is handled naturally as permits are released
    }
    
    /// Get current statistics
    pub fn stats(&self) -> DownloadStats {
        DownloadStats {
            active: self.active_count(),
            queued: self.queue_len(),
            max_concurrent: *self.inner.max_concurrent.read(),
        }
    }
}

/// Download statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct DownloadStats {
    pub active: usize,
    pub queued: usize,
    pub max_concurrent: usize,
}

/// Extract filename from URL
pub fn extract_filename(url: &str, content_disposition: Option<&str>) -> String {
    // Try Content-Disposition header first
    if let Some(cd) = content_disposition {
        if let Some(start) = cd.find("filename=") {
            let start = start + 9;
            let filename = &cd[start..];
            let filename = filename.trim_matches('"').trim_matches('\'');
            if !filename.is_empty() {
                return filename.to_string();
            }
        }
    }
    
    // Fall back to URL path
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(segments) = parsed.path_segments() {
            if let Some(last) = segments.last() {
                let decoded = urlencoding::decode(last).unwrap_or_else(|_| last.into());
                if !decoded.is_empty() && decoded != "/" {
                    return decoded.to_string();
                }
            }
        }
    }
    
    // Last resort: generate a name
    format!("download_{}", chrono::Utc::now().timestamp())
}
