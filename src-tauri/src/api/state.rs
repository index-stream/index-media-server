use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::AppHandle;

// Shared state for HTTP server
pub type AppState = Arc<Mutex<Option<AppHandle>>>;

// Extended state that includes HTTPS port information
#[derive(Clone)]
pub struct ServerState {
    pub app_handle: Option<AppHandle>,
    pub https_port: Option<u16>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            app_handle: None,
            https_port: None,
        }
    }
}

pub type ExtendedAppState = Arc<Mutex<ServerState>>;
