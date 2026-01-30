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

/// Check for required system dependencies on Linux
#[cfg(target_os = "linux")]
fn check_linux_dependencies() {
    use std::process::Command;
    
    let libs = [
        ("libgtk-3.so.0", "libgtk-3-0"),
        ("libayatana-appindicator3.so.1", "libayatana-appindicator3-1"),
        ("libxdo.so.3", "libxdo3"),
    ];
    
    let mut missing = Vec::new();
    
    for (lib, package) in &libs {
        // Use ldconfig to check if library is available
        let output = Command::new("ldconfig")
            .args(["-p"])
            .output();
        
        let found = match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.contains(lib)
            }
            Err(_) => {
                // Fallback: check common library paths
                std::path::Path::new(&format!("/usr/lib/x86_64-linux-gnu/{}", lib)).exists()
                    || std::path::Path::new(&format!("/usr/lib/{}", lib)).exists()
            }
        };
        
        if !found {
            missing.push(*package);
        }
    }
    
    if !missing.is_empty() {
        eprintln!("\n╭─────────────────────────────────────────────────────────────╮");
        eprintln!("│  ⚠️  Missing Dependencies                                    │");
        eprintln!("├─────────────────────────────────────────────────────────────┤");
        eprintln!("│  The following packages are required but not installed:     │");
        for pkg in &missing {
            eprintln!("│    • {:<53} │", pkg);
        }
        eprintln!("├─────────────────────────────────────────────────────────────┤");
        eprintln!("│  Install with:                                              │");
        eprintln!("│    sudo apt install {}  │", missing.join(" "));
        eprintln!("╰─────────────────────────────────────────────────────────────╯\n");
    }
}

#[cfg(not(target_os = "linux"))]
fn check_linux_dependencies() {}

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
    // Check for required dependencies on Linux
    check_linux_dependencies();
    
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
