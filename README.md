# Vibe Downloader

A cross-platform download manager with a web interface accessible from any device on your local network.

## Features

- 🌐 **Web UI** - Access from any device on your LAN
- 📂 **File Type Categories** - Automatically organize downloads by type
- ⚡ **Concurrent Downloads** - Configurable download queue with concurrency limit
- 🔄 **Real-time Progress** - WebSocket-powered live updates
- 🖥️ **System Tray** - Runs in background with tray icon
- 🚀 **Auto-start** - Optionally start on system login
- 🦀 **Cross-platform** - Works on Windows, Linux, and macOS

## Architecture

- **Backend**: Rust (Axum web framework)
- **Frontend**: React + TypeScript + Tailwind CSS
- **Database**: SQLite (embedded)
- **Real-time**: WebSocket for progress updates

## Development

### Prerequisites

- Rust (1.75+)
- Node.js (20+)
- pnpm or npm

### Running in Development

1. **Start the frontend dev server:**
   ```bash
   cd frontend
   npm install
   npm run dev
   ```

2. **Start the backend:**
   ```bash
   cd backend
   cargo run
   ```

The frontend runs on `http://localhost:5173` and proxies API requests to the backend on port 8787.

### Building for Production

1. **Build the frontend:**
   ```bash
   cd frontend
   npm run build
   ```

2. **Build the backend (with embedded frontend):**
   ```bash
   cd backend
   cargo build --release
   ```

The release binary at `target/release/vibe-downloader` contains everything - no additional files needed.

## Configuration

Configuration is stored in:
- **Windows**: `%APPDATA%\vibe-downloader\config.toml`
- **macOS**: `~/Library/Application Support/vibe-downloader/config.toml`
- **Linux**: `~/.config/vibe-downloader/config.toml`

### Default Settings

```toml
[server]
host = "0.0.0.0"
port = 8787

max_concurrent_downloads = 3
start_on_login = false

[file_types.general]
name = "General"
extensions = ["*"]
destination = "~/Downloads"

[file_types.video]
name = "Video"
extensions = ["mp4", "mkv", "avi", "mov", "webm"]
destination = "~/Downloads/Videos"

# ... more file types
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/downloads` | List all downloads |
| POST | `/api/downloads` | Add a new download |
| DELETE | `/api/downloads/:id` | Remove a download |
| POST | `/api/downloads/:id/cancel` | Cancel an active download |
| GET | `/api/downloads/stats` | Get download statistics |
| GET | `/api/settings` | Get current settings |
| PUT | `/api/settings` | Update settings |
| GET | `/api/file-types` | List file type configurations |
| POST | `/api/file-types` | Add a file type |
| PUT | `/api/file-types/:id` | Update a file type |
| DELETE | `/api/file-types/:id` | Remove a file type |
| WS | `/ws` | WebSocket for real-time progress |

## License

MIT
