use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::AppHandle;
use sqlx::SqlitePool;

// Unified app state containing both database pool and HTTPS port information
#[derive(Clone)]
pub struct AppState {
    pub app_handle: Arc<Mutex<Option<AppHandle>>>,
    pub db_pool: SqlitePool,
    pub https_port: Arc<Mutex<Option<u16>>>,
}
