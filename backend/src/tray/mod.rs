//! System tray module for background running

use crate::AppState;
use anyhow::Result;
use std::sync::Arc;
use tray_icon::{
    menu::{Menu, MenuItem},
    TrayIconBuilder,
};
use tracing::info;

/// Run the system tray
pub fn run(state: Arc<AppState>) -> Result<()> {
    // Create tray menu
    let menu = Menu::new();
    
    let open_item = MenuItem::new("Open Web UI", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    
    let open_id = open_item.id().clone();
    let quit_id = quit_item.id().clone();
    
    menu.append(&open_item)?;
    menu.append(&quit_item)?;
    
    // Create tray icon
    let icon = load_icon()?;
    
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Vibe Downloader")
        .with_icon(icon)
        .build()?;
    
    info!("System tray initialized");
    
    // Get server URL for opening
    let port = state.settings.read().server.port;
    let url = format!("http://localhost:{}", port);
    
    // Event loop
    let event_loop = tray_icon::menu::MenuEvent::receiver();
    
    loop {
        if let Ok(event) = event_loop.recv() {
            if event.id == open_id {
                info!("Opening web UI: {}", url);
                let _ = open::that(&url);
            } else if event.id == quit_id {
                info!("Quit requested from tray menu");
                std::process::exit(0);
            }
        }
    }
}

/// Load the tray icon
fn load_icon() -> Result<tray_icon::Icon> {
    // Create a simple colored icon programmatically
    // In production, you'd load from a file
    let size = 32u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    
    for y in 0..size {
        for x in 0..size {
            // Create a simple gradient icon (blue to purple)
            let r = ((x as f32 / size as f32) * 100.0 + 50.0) as u8;
            let g = 100u8;
            let b = ((y as f32 / size as f32) * 100.0 + 155.0) as u8;
            let a = 255u8;
            
            // Make it circular
            let cx = size as f32 / 2.0;
            let cy = size as f32 / 2.0;
            let dist = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
            
            if dist <= size as f32 / 2.0 - 1.0 {
                rgba.extend_from_slice(&[r, g, b, a]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    
    let icon = tray_icon::Icon::from_rgba(rgba, size, size)?;
    Ok(icon)
}
