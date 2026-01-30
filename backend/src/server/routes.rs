//! REST API routes

use crate::config::{self, FileTypeConfig};
use crate::db::{DownloadRecord, DownloadStatus};
use crate::download::{self, DownloadStats};
use crate::AppState;
use auto_launch::AutoLaunchBuilder;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use futures_util::StreamExt;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;

/// Create API routes
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Downloads
        .route("/downloads", get(list_downloads))
        .route("/downloads", post(add_download))
        .route("/downloads/{id}", delete(remove_download))
        .route("/downloads/{id}/cancel", post(cancel_download))
        .route("/downloads/stats", get(download_stats))
        // Settings
        .route("/settings", get(get_settings))
        .route("/settings", put(update_settings))
        // File types
        .route("/file-types", get(list_file_types))
        .route("/file-types", post(add_file_type))
        .route("/file-types/{id}", put(update_file_type))
        .route("/file-types/{id}", delete(remove_file_type))
}

/// Resume incomplete downloads from previous session
pub fn resume_incomplete_downloads(state: Arc<AppState>) {
    let downloads = match state.db.get_all_downloads() {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("Failed to load downloads for resume: {}", e);
            return;
        }
    };
    
    let max_concurrent = state.settings.read().max_concurrent_downloads;
    let mut started = 0;
    
    for download in downloads {
        match download.status {
            DownloadStatus::Downloading | DownloadStatus::Pending => {
                // These were interrupted - restart them
                if started < max_concurrent {
                    info!("Resuming download: {}", download.filename);
                    start_download(state.clone(), download);
                    started += 1;
                } else {
                    // Queue the rest
                    info!("Queueing download: {}", download.filename);
                    let _ = state.db.update_status(&download.id, DownloadStatus::Queued, None);
                    state.download_manager.enqueue(download);
                }
            }
            DownloadStatus::Queued => {
                // Re-enqueue
                if started < max_concurrent {
                    info!("Starting queued download: {}", download.filename);
                    start_download(state.clone(), download);
                    started += 1;
                } else {
                    state.download_manager.enqueue(download);
                }
            }
            _ => {} // Completed, Failed, Cancelled - leave as is
        }
    }
    
    if started > 0 {
        info!("Resumed {} downloads", started);
    }
}

// ============ Download Endpoints ============

/// List all downloads
async fn list_downloads(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DownloadRecord>>, AppError> {
    let downloads = state.db.get_all_downloads()?;
    Ok(Json(downloads))
}

/// Request to add a new download
#[derive(Debug, Deserialize)]
pub struct AddDownloadRequest {
    pub url: String,
    pub file_type: String,
    pub filename: Option<String>,
}

/// Response after adding a download
#[derive(Debug, Serialize)]
pub struct AddDownloadResponse {
    pub id: String,
    pub queued: bool,
}

/// Add a new download
async fn add_download(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddDownloadRequest>,
) -> Result<Json<AddDownloadResponse>, AppError> {
    let settings = state.settings.read().clone();
    
    // Get destination folder from file type
    let file_type_config = settings
        .file_types
        .get(&req.file_type)
        .or_else(|| settings.file_types.get("general"))
        .ok_or_else(|| AppError::BadRequest("Unknown file type".into()))?;
    
    // Extract filename from URL if not provided
    let filename = req.filename.unwrap_or_else(|| {
        download::extract_filename(&req.url, None)
    });
    
    // Create download record
    let record = DownloadRecord::new(
        req.url.clone(),
        filename,
        req.file_type.clone(),
        file_type_config.destination.clone(),
    );
    
    let id = record.id.clone();
    
    // Insert into database
    state.db.insert_download(&record)?;
    
    // Check if we should queue or start immediately
    let active_count = state.download_manager.active_count();
    let max_concurrent = state.settings.read().max_concurrent_downloads;
    let queued = active_count >= max_concurrent;
    
    if queued {
        // Update status to queued
        state.db.update_status(&id, DownloadStatus::Queued, None)?;
        state.download_manager.enqueue(record);
    } else {
        // Start download immediately
        start_download(state.clone(), record);
    }
    
    Ok(Json(AddDownloadResponse { id, queued }))
}

/// Start a download task
fn start_download(state: Arc<AppState>, record: DownloadRecord) {
    let db = state.db.clone();
    let download_manager = state.download_manager.clone();
    let settings = state.settings.read().clone();
    let progress_tx = download_manager.progress_sender();
    
    // Create cancel channel
    let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);
    
    // Register as active
    let download_id = record.id.clone();
    download_manager.add_active(download_id.clone(), cancel_tx);
    
    // Update status to downloading
    let _ = db.update_status(&record.id, DownloadStatus::Downloading, None);
    
    // Send initial progress update
    let _ = progress_tx.send(download::ProgressUpdate {
        id: record.id.clone(),
        downloaded: 0,
        total: record.total_size,
        speed: 0,
        status: DownloadStatus::Downloading,
        error: None,
    });
    
    tokio::spawn(async move {
        // Perform download with cancellation support
        let result = download_file_with_cancel(&record, &progress_tx, &mut cancel_rx).await;
        
        // Remove from active set
        download_manager.remove_active(&record.id);
        
        match result {
            Ok(_) => {
                let _ = db.update_status(&record.id, DownloadStatus::Completed, None);
                let _ = progress_tx.send(download::ProgressUpdate {
                    id: record.id.clone(),
                    downloaded: record.total_size.unwrap_or(0),
                    total: record.total_size,
                    speed: 0,
                    status: DownloadStatus::Completed,
                    error: None,
                });
            }
            Err(e) => {
                let error_msg = e.to_string();
                let status = if error_msg.contains("cancelled") {
                    DownloadStatus::Cancelled
                } else {
                    DownloadStatus::Failed
                };
                let _ = db.update_status(&record.id, status.clone(), Some(&error_msg));
                let _ = progress_tx.send(download::ProgressUpdate {
                    id: record.id.clone(),
                    downloaded: 0,
                    total: None,
                    speed: 0,
                    status,
                    error: Some(error_msg),
                });
            }
        }
        
        // Try to start next queued download
        if let Some(next) = download_manager.dequeue() {
            // Rebuild a minimal state for the next download
            let next_state = Arc::new(AppState {
                settings: RwLock::new(settings),
                db: db.clone(),
                download_manager: download_manager.clone(),
            });
            start_download(next_state, next);
        }
    });
}

/// Remove a download
async fn remove_download(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    // Cancel if active
    state.download_manager.cancel(&id).await;
    
    // Remove from database
    if let Err(e) = state.db.delete_download(&id) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response();
    }
    
    StatusCode::NO_CONTENT.into_response()
}

/// Cancel an active download
async fn cancel_download(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Response {
    if state.download_manager.cancel(&id).await {
        (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "Download not found or already completed" }))).into_response()
    }
}

/// Get download statistics
async fn download_stats(
    State(state): State<Arc<AppState>>,
) -> Json<DownloadStats> {
    Json(state.download_manager.stats())
}

// ============ Settings Endpoints ============

/// Settings response (excluding sensitive data)
#[derive(Debug, Serialize)]
pub struct SettingsResponse {
    pub server_port: u16,
    pub max_concurrent_downloads: usize,
    pub start_on_login: bool,
}

/// Get current settings
async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Json<SettingsResponse> {
    let settings = state.settings.read();
    Json(SettingsResponse {
        server_port: settings.server.port,
        max_concurrent_downloads: settings.max_concurrent_downloads,
        start_on_login: settings.start_on_login,
    })
}

/// Update settings request
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub max_concurrent_downloads: Option<usize>,
    pub start_on_login: Option<bool>,
}

/// Update settings
async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<SettingsResponse>, AppError> {
    let mut settings = state.settings.write();
    
    if let Some(max) = req.max_concurrent_downloads {
        settings.max_concurrent_downloads = max;
        state.download_manager.set_max_concurrent(max);
    }
    
    if let Some(start) = req.start_on_login {
        settings.start_on_login = start;
        
        // Configure auto-launch
        if let Err(e) = configure_auto_launch(start) {
            tracing::error!("Failed to configure auto-launch: {}", e);
        }
    }
    
    // Save to file
    config::save(&settings)?;
    
    Ok(Json(SettingsResponse {
        server_port: settings.server.port,
        max_concurrent_downloads: settings.max_concurrent_downloads,
        start_on_login: settings.start_on_login,
    }))
}

/// Configure auto-launch on system startup
fn configure_auto_launch(enable: bool) -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    let exe_path_str = exe_path.to_string_lossy().to_string();
    
    let auto_launch = AutoLaunchBuilder::new()
        .set_app_name("Vibe Downloader")
        .set_app_path(&exe_path_str)
        .set_use_launch_agent(true) // macOS: use LaunchAgent instead of login items
        .build()
        .map_err(|e| format!("Failed to build auto-launch: {}", e))?;
    
    if enable {
        auto_launch.enable().map_err(|e| format!("Failed to enable auto-launch: {}", e))?;
        info!("Auto-launch enabled");
    } else {
        // Only disable if currently enabled
        if auto_launch.is_enabled().unwrap_or(false) {
            auto_launch.disable().map_err(|e| format!("Failed to disable auto-launch: {}", e))?;
            info!("Auto-launch disabled");
        }
    }
    
    Ok(())
}

// ============ File Type Endpoints ============

/// List all file types
async fn list_file_types(
    State(state): State<Arc<AppState>>,
) -> Json<HashMap<String, FileTypeConfig>> {
    let settings = state.settings.read();
    Json(settings.file_types.clone())
}

/// Add file type request
#[derive(Debug, Deserialize)]
pub struct AddFileTypeRequest {
    pub name: String,
    pub extensions: Vec<String>,
    pub destination: String,
}

/// Add a new file type
async fn add_file_type(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddFileTypeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut settings = state.settings.write();
    
    // Generate unique ID from name + timestamp to allow multiple categories
    let base_id = req.name.to_lowercase().replace(|c: char| !c.is_alphanumeric(), "-");
    let mut id = base_id.clone();
    let mut counter = 1;
    
    // If ID exists, append a number to make it unique
    while settings.file_types.contains_key(&id) {
        id = format!("{}-{}", base_id, counter);
        counter += 1;
    }
    
    settings.file_types.insert(
        id.clone(),
        FileTypeConfig {
            name: req.name,
            extensions: req.extensions,
            destination: PathBuf::from(req.destination),
        },
    );
    
    config::save(&settings)?;
    
    Ok(Json(serde_json::json!({ "id": id })))
}

/// Update file type request
#[derive(Debug, Deserialize)]
pub struct UpdateFileTypeRequest {
    pub name: Option<String>,
    pub extensions: Option<Vec<String>>,
    pub destination: Option<String>,
}

/// Update an existing file type
async fn update_file_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateFileTypeRequest>,
) -> Result<StatusCode, AppError> {
    let mut settings = state.settings.write();
    
    let file_type = settings
        .file_types
        .get_mut(&id)
        .ok_or_else(|| AppError::NotFound("File type not found".into()))?;
    
    if let Some(name) = req.name {
        file_type.name = name;
    }
    if let Some(extensions) = req.extensions {
        file_type.extensions = extensions;
    }
    if let Some(destination) = req.destination {
        file_type.destination = PathBuf::from(destination);
    }
    
    config::save(&settings)?;
    
    Ok(StatusCode::OK)
}

/// Remove a file type
async fn remove_file_type(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut settings = state.settings.write();
    
    if id == "general" {
        return Err(AppError::BadRequest("Cannot remove default file type".into()));
    }
    
    if settings.file_types.remove(&id).is_none() {
        return Err(AppError::NotFound("File type not found".into()));
    }
    
    config::save(&settings)?;
    
    Ok(StatusCode::NO_CONTENT)
}

// ============ Error Handling ============

/// Application error type
#[derive(Debug)]
pub enum AppError {
    Internal(String),
    BadRequest(String),
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };
        
        let body = Json(serde_json::json!({ "error": message }));
        (status, body).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

/// Download file with cancellation support
async fn download_file_with_cancel(
    record: &DownloadRecord,
    progress_tx: &tokio::sync::broadcast::Sender<download::ProgressUpdate>,
    cancel_rx: &mut tokio::sync::mpsc::Receiver<()>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("VibeDownloader/1.0")
        .build()?;
    
    let response = client.get(&record.url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }
    
    let total_size = response.content_length();
    
    // Ensure destination directory exists
    tokio::fs::create_dir_all(&record.destination).await?;
    
    // Use .part extension while downloading
    let final_path = record.destination.join(&record.filename);
    let temp_path = record.destination.join(format!("{}.part", &record.filename));
    let mut file = File::create(&temp_path).await?;
    
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let start_time = std::time::Instant::now();
    let mut last_update = std::time::Instant::now();
    
    loop {
        tokio::select! {
            // Check for cancellation
            _ = cancel_rx.recv() => {
                // Clean up partial file
                drop(file);
                let _ = tokio::fs::remove_file(&temp_path).await;
                anyhow::bail!("Download cancelled");
            }
            // Process next chunk
            chunk = stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        file.write_all(&bytes).await?;
                        downloaded += bytes.len() as u64;
                        
                        // Send progress every 200ms
                        if last_update.elapsed().as_millis() >= 200 {
                            let elapsed = start_time.elapsed().as_secs_f64();
                            let speed = if elapsed > 0.0 { (downloaded as f64 / elapsed) as u64 } else { 0 };
                            
                            let _ = progress_tx.send(download::ProgressUpdate {
                                id: record.id.clone(),
                                downloaded,
                                total: total_size,
                                speed,
                                status: DownloadStatus::Downloading,
                                error: None,
                            });
                            last_update = std::time::Instant::now();
                        }
                    }
                    Some(Err(e)) => {
                        // Clean up on error
                        drop(file);
                        let _ = tokio::fs::remove_file(&temp_path).await;
                        anyhow::bail!("Download error: {}", e);
                    }
                    None => break, // Stream ended
                }
            }
        }
    }
    
    file.flush().await?;
    drop(file);
    
    // Rename from .part to final filename
    tokio::fs::rename(&temp_path, &final_path).await?;
    
    Ok(())
}
