//! Vibe Downloader - A cross-platform download manager with web UI
//!
//! This application runs as a background service with a system tray icon,
//! serving a web interface accessible from any device on the LAN.

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod config;
mod db;
mod download;
mod server;
mod tray;

use anyhow::Result;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::config::Settings;
use crate::db::Database;
use crate::download::DownloadManager;

/// Application state shared across all components
pub struct AppState {
    pub settings: RwLock<Settings>,
    pub db: Database,
    pub download_manager: DownloadManager,
}

impl AppState {
    pub fn new(settings: Settings, db: Database) -> Self {
        let download_manager = DownloadManager::new(settings.max_concurrent_downloads);
        Self {
            settings: RwLock::new(settings),
            db,
            download_manager,
        }
    }
}

fn main() -> Result<()> {
    // Initialize logging
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();

    info!("Starting Vibe Downloader v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let settings = config::load_or_create_default()?;
    info!("Configuration loaded from {:?}", config::config_path());

    // Initialize database
    let db = Database::new()?;
    info!("Database initialized");

    // Create shared application state
    let state = Arc::new(AppState::new(settings, db));

    // Start the async runtime for the server
    let server_state = Arc::clone(&state);
    let server_handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            if let Err(e) = server::run(server_state).await {
                tracing::error!("Server error: {}", e);
            }
        });
    });

    // Run the system tray on the main thread (required by most platforms)
    info!("Starting system tray...");
    tray::run(Arc::clone(&state))?;

    // Wait for server thread to finish (it won't unless there's an error)
    let _ = server_handle.join();

    Ok(())
}
