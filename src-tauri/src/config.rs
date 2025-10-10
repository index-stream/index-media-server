use tauri::{path::BaseDirectory, Manager};
use anyhow::Result;

/// Get the unified app data directory using Tauri's app data directory
pub fn get_app_data_dir(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf> {
    let dir = app_handle
        .path()
        .resolve("data", BaseDirectory::AppData)?; // e.g., ~/Library/Application Support/com.kalegd.index-media-server/data
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Get the SQLite database path using Tauri's app data directory
pub fn sqlite_path(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf> {
    Ok(get_app_data_dir(app_handle)?.join("app.sqlite3"))
}

/// Get the config.json path using Tauri's app data directory
pub fn config_path(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf> {
    Ok(get_app_data_dir(app_handle)?.join("config.json"))
}

/// Get the icons directory path using Tauri's app data directory
pub fn icons_dir(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf> {
    let icons_dir = get_app_data_dir(app_handle)?.join("icons");
    std::fs::create_dir_all(&icons_dir)?;
    Ok(icons_dir)
}

/// Get the certificates directory path using Tauri's app data directory
pub fn certs_dir(app_handle: &tauri::AppHandle) -> Result<std::path::PathBuf> {
    let certs_dir = get_app_data_dir(app_handle)?.join("certs");
    std::fs::create_dir_all(&certs_dir)?;
    Ok(certs_dir)
}
