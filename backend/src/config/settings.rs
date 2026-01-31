//! Settings data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Server configuration
    pub server: ServerSettings,
    
    /// Maximum number of concurrent downloads
    pub max_concurrent_downloads: usize,
    
    /// File type to destination folder mappings
    pub file_types: HashMap<String, FileTypeConfig>,
    
    /// Whether to start on system login
    pub start_on_login: bool,
    
    /// Whether to start on boot without login (Linux systemd service)
    #[serde(default)]
    pub start_on_boot: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let mut file_types = HashMap::new();
        
        // Default file type mappings
        let downloads_dir = dirs::download_dir()
            .unwrap_or_else(|| PathBuf::from("./downloads"));
        
        file_types.insert(
            "general".to_string(),
            FileTypeConfig {
                name: "General".to_string(),
                extensions: vec!["*".to_string()],
                destination: downloads_dir.clone(),
            },
        );
        
        file_types.insert(
            "video".to_string(),
            FileTypeConfig {
                name: "Video".to_string(),
                extensions: vec![
                    "mp4".to_string(),
                    "mkv".to_string(),
                    "avi".to_string(),
                    "mov".to_string(),
                    "webm".to_string(),
                ],
                destination: downloads_dir.join("Videos"),
            },
        );
        
        file_types.insert(
            "audio".to_string(),
            FileTypeConfig {
                name: "Audio".to_string(),
                extensions: vec![
                    "mp3".to_string(),
                    "flac".to_string(),
                    "wav".to_string(),
                    "aac".to_string(),
                    "ogg".to_string(),
                ],
                destination: downloads_dir.join("Audio"),
            },
        );
        
        file_types.insert(
            "documents".to_string(),
            FileTypeConfig {
                name: "Documents".to_string(),
                extensions: vec![
                    "pdf".to_string(),
                    "doc".to_string(),
                    "docx".to_string(),
                    "txt".to_string(),
                    "xlsx".to_string(),
                ],
                destination: downloads_dir.join("Documents"),
            },
        );
        
        file_types.insert(
            "images".to_string(),
            FileTypeConfig {
                name: "Images".to_string(),
                extensions: vec![
                    "jpg".to_string(),
                    "jpeg".to_string(),
                    "png".to_string(),
                    "gif".to_string(),
                    "webp".to_string(),
                    "svg".to_string(),
                ],
                destination: downloads_dir.join("Images"),
            },
        );
        
        file_types.insert(
            "archives".to_string(),
            FileTypeConfig {
                name: "Archives".to_string(),
                extensions: vec![
                    "zip".to_string(),
                    "rar".to_string(),
                    "7z".to_string(),
                    "tar".to_string(),
                    "gz".to_string(),
                ],
                destination: downloads_dir.join("Archives"),
            },
        );

        Self {
            server: ServerSettings::default(),
            max_concurrent_downloads: 3,
            file_types,
            start_on_login: false,
            start_on_boot: false,
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// Host to bind to (0.0.0.0 for LAN access)
    pub host: String,
    
    /// Port to listen on
    pub port: u16,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8787,
        }
    }
}

/// Configuration for a file type category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeConfig {
    /// Display name for the file type
    pub name: String,
    
    /// File extensions that belong to this category
    pub extensions: Vec<String>,
    
    /// Destination folder for downloads of this type
    pub destination: PathBuf,
}
