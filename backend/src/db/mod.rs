//! Database module for persisting download history and state

mod schema;

pub use schema::*;

use anyhow::Result;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Database wrapper for SQLite operations
#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    pub fn new() -> Result<Self> {
        let path = Self::db_path();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(&path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db.initialize_schema()?;
        
        Ok(db)
    }
    
    /// Get the database file path
    fn db_path() -> PathBuf {
        crate::config::config_dir().join("downloads.db")
    }
    
    /// Initialize the database schema
    fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS downloads (
                id TEXT PRIMARY KEY,
                url TEXT NOT NULL,
                filename TEXT NOT NULL,
                file_type TEXT NOT NULL,
                destination TEXT NOT NULL,
                total_size INTEGER,
                downloaded_size INTEGER DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'pending',
                error_message TEXT,
                created_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT
            );
            
            CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status);
            CREATE INDEX IF NOT EXISTS idx_downloads_created_at ON downloads(created_at);
            "#,
        )?;
        
        Ok(())
    }
    
    /// Insert a new download record
    pub fn insert_download(&self, download: &DownloadRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            r#"
            INSERT INTO downloads (
                id, url, filename, file_type, destination, 
                total_size, downloaded_size, status, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            rusqlite::params![
                download.id,
                download.url,
                download.filename,
                download.file_type,
                download.destination.to_string_lossy(),
                download.total_size,
                download.downloaded_size,
                download.status.as_str(),
                download.created_at.to_rfc3339(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Update download progress
    pub fn update_progress(&self, id: &str, downloaded: u64, total: Option<u64>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "UPDATE downloads SET downloaded_size = ?1, total_size = ?2 WHERE id = ?3",
            rusqlite::params![downloaded, total, id],
        )?;
        
        Ok(())
    }
    
    /// Update download status
    pub fn update_status(&self, id: &str, status: DownloadStatus, error: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        let now = chrono::Utc::now().to_rfc3339();
        
        match status {
            DownloadStatus::Downloading => {
                conn.execute(
                    "UPDATE downloads SET status = ?1, started_at = ?2 WHERE id = ?3",
                    rusqlite::params![status.as_str(), now, id],
                )?;
            }
            DownloadStatus::Completed | DownloadStatus::Failed => {
                conn.execute(
                    "UPDATE downloads SET status = ?1, completed_at = ?2, error_message = ?3 WHERE id = ?4",
                    rusqlite::params![status.as_str(), now, error, id],
                )?;
            }
            _ => {
                conn.execute(
                    "UPDATE downloads SET status = ?1, error_message = ?2 WHERE id = ?3",
                    rusqlite::params![status.as_str(), error, id],
                )?;
            }
        }
        
        Ok(())
    }
    
    /// Get all downloads
    pub fn get_all_downloads(&self) -> Result<Vec<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, url, filename, file_type, destination, 
                   total_size, downloaded_size, status, error_message,
                   created_at, started_at, completed_at
            FROM downloads
            ORDER BY created_at DESC
            "#,
        )?;
        
        let downloads = stmt
            .query_map([], |row| {
                Ok(DownloadRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    filename: row.get(2)?,
                    file_type: row.get(3)?,
                    destination: PathBuf::from(row.get::<_, String>(4)?),
                    total_size: row.get(5)?,
                    downloaded_size: row.get(6)?,
                    status: DownloadStatus::from_str(&row.get::<_, String>(7)?),
                    error_message: row.get(8)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    started_at: row.get::<_, Option<String>>(10)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    completed_at: row.get::<_, Option<String>>(11)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        
        Ok(downloads)
    }
    
    /// Delete a download record
    pub fn delete_download(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM downloads WHERE id = ?1", [id])?;
        Ok(())
    }
    
    /// Get pending downloads (for resuming on startup)
    pub fn get_pending_downloads(&self) -> Result<Vec<DownloadRecord>> {
        let conn = self.conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            r#"
            SELECT id, url, filename, file_type, destination, 
                   total_size, downloaded_size, status, error_message,
                   created_at, started_at, completed_at
            FROM downloads
            WHERE status IN ('pending', 'queued', 'downloading')
            ORDER BY created_at ASC
            "#,
        )?;
        
        let downloads = stmt
            .query_map([], |row| {
                Ok(DownloadRecord {
                    id: row.get(0)?,
                    url: row.get(1)?,
                    filename: row.get(2)?,
                    file_type: row.get(3)?,
                    destination: PathBuf::from(row.get::<_, String>(4)?),
                    total_size: row.get(5)?,
                    downloaded_size: row.get(6)?,
                    status: DownloadStatus::from_str(&row.get::<_, String>(7)?),
                    error_message: row.get(8)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                    started_at: row.get::<_, Option<String>>(10)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    completed_at: row.get::<_, Option<String>>(11)?
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        
        Ok(downloads)
    }
}
