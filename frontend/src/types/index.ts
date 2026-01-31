// API types matching the Rust backend

export interface DownloadRecord {
  id: string
  url: string
  filename: string
  file_type: string
  destination: string
  total_size: number | null
  downloaded_size: number
  status: DownloadStatus
  error_message: string | null
  created_at: string
  started_at: string | null
  completed_at: string | null
}

export type DownloadStatus = 
  | 'pending'
  | 'queued'
  | 'downloading'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'cancelled'

export interface DownloadStats {
  active: number
  queued: number
  max_concurrent: number
}

export interface FileTypeConfig {
  name: string
  extensions: string[]
  destination: string
}

export interface Settings {
  server_port: number
  max_concurrent_downloads: number
  start_on_login: boolean
  start_on_boot: boolean
  start_on_boot_available: boolean
}

export interface ProgressUpdate {
  id: string
  downloaded: number
  total: number | null
  speed: number
  status: DownloadStatus
  error: string | null
}

export interface AddDownloadRequest {
  url: string
  file_type: string
  filename?: string
}

export interface AddDownloadResponse {
  id: string
  queued: boolean
}
