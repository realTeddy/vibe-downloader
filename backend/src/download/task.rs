//! Individual download task implementation

use crate::db::{DownloadRecord, DownloadStatus};
use crate::download::ProgressUpdate;
use anyhow::Result;
use futures_util::StreamExt;
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::{broadcast, mpsc, Semaphore};
use tracing::info;

/// Download context for running a download task
pub struct DownloadContext {
    pub semaphore: Arc<Semaphore>,
    pub progress_tx: broadcast::Sender<ProgressUpdate>,
    pub cancel_rx: mpsc::Receiver<()>,
}

/// Run a download with the given context
pub async fn run_download(
    record: &DownloadRecord,
    ctx: &mut DownloadContext,
) -> Result<()> {
    // Acquire semaphore permit
    let semaphore = Arc::clone(&ctx.semaphore);
    let _permit = semaphore.acquire().await?;
    
    info!("Starting download: {} -> {}", record.url, record.filename);
    
    // Perform the download
    download_file(record, &mut ctx.cancel_rx, &ctx.progress_tx).await
}

/// Download a file with progress tracking
pub async fn download_file(
    record: &DownloadRecord,
    cancel_rx: &mut mpsc::Receiver<()>,
    progress_tx: &broadcast::Sender<ProgressUpdate>,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("VibeDownloader/1.0")
        .build()?;
    
    let response = client.get(&record.url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }
    
    let total_size = response.content_length();
    
    // Ensure destination directory exists
    if let Some(parent) = record.destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    
    let file_path = record.destination.join(&record.filename);
    let mut file = File::create(&file_path).await?;
    
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let start_time = Instant::now();
    let mut last_progress_time = Instant::now();
    
    loop {
        tokio::select! {
            // Check for cancellation
            _ = cancel_rx.recv() => {
                // Clean up partial file
                drop(file);
                let _ = tokio::fs::remove_file(&file_path).await;
                anyhow::bail!("Download cancelled");
            }
            
            // Process next chunk
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        file.write_all(&bytes).await?;
                        downloaded += bytes.len() as u64;
                        
                        // Update progress every 100ms
                        if last_progress_time.elapsed().as_millis() >= 100 {
                            let elapsed = start_time.elapsed().as_secs_f64();
                            let speed = if elapsed > 0.0 {
                                (downloaded as f64 / elapsed) as u64
                            } else {
                                0
                            };
                            
                            // Send progress update
                            let _ = progress_tx.send(ProgressUpdate {
                                id: record.id.clone(),
                                downloaded,
                                total: total_size,
                                speed,
                                status: DownloadStatus::Downloading,
                                error: None,
                            });
                            
                            last_progress_time = Instant::now();
                        }
                    }
                    Some(Err(e)) => {
                        // Clean up partial file
                        drop(file);
                        let _ = tokio::fs::remove_file(&file_path).await;
                        anyhow::bail!("Download error: {}", e);
                    }
                    None => {
                        // Download complete
                        break;
                    }
                }
            }
        }
    }
    
    file.flush().await?;
    
    Ok(())
}

/// Extract filename from URL or Content-Disposition header
pub fn extract_filename(url: &str, content_disposition: Option<&str>) -> String {
    // Try Content-Disposition header first
    if let Some(cd) = content_disposition {
        if let Some(start) = cd.find("filename=") {
            let name = &cd[start + 9..];
            let name = name.trim_matches('"').trim_matches('\'');
            if let Some(end) = name.find(';') {
                return name[..end].to_string();
            }
            return name.to_string();
        }
    }
    
    // Fall back to URL path
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(segments) = parsed.path_segments() {
            if let Some(last) = segments.last() {
                if !last.is_empty() {
                    return urlencoding::decode(last)
                        .map(|s| s.into_owned())
                        .unwrap_or_else(|_| last.to_string());
                }
            }
        }
    }
    
    // Last resort: generate a name
    format!("download_{}", uuid::Uuid::new_v4())
}
