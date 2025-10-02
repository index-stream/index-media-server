# Index Media Server - Rust Backend

This directory contains the Rust backend for the Index Media Server application.

## File Structure

```
src/
├── main.rs          # Main application entry point and HTTP server setup
├── lib.rs           # Library module with re-exports
├── http.rs          # HTTP server functionality (static file serving)
├── dialog.rs         # Dialog functionality (folder selection)
└── README.md         # This file
```

## Modules

### `main.rs`
- Application entry point
- HTTP server setup and routing
- Tauri app configuration
- Tray icon and menu setup

### `lib.rs`
- Library module definition
- Re-exports commonly used types and functions
- Provides clean API for other modules

### `http.rs`
- Static file serving with SPA fallback
- Content type detection
- File system operations for web assets

### `dialog.rs`
- Multiple folder selection dialog functionality
- Temporary window management for dialog positioning
- HTTP API handlers for dialog operations
- Redundant folder filtering (removes child folders when parent is selected)

## Key Features

- **Static File Serving**: Serves web assets with proper MIME types
- **SPA Support**: Fallback to index.html for client-side routing
- **Multiple Folder Selection**: Native dialog supports selecting multiple folders at once
- **Smart Folder Filtering**: Automatically removes redundant child folders when parent folders are selected
- **HTTP API**: RESTful endpoints for frontend communication
- **Tray Integration**: System tray icon with menu

## Architecture

The application uses a modular architecture:
- **Separation of Concerns**: Each module handles specific functionality
- **Clean APIs**: Well-defined interfaces between modules
- **Reusable Components**: Functions can be easily tested and reused
- **Type Safety**: Strong typing throughout the codebase
