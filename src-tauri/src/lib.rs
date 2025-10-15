// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

pub mod api;
pub mod models;
pub mod utils;
pub mod constants;
pub mod db;
pub mod config;
pub mod scanning;

// Re-export commonly used types and functions
pub use api::folders::{handle_select_folders, select_folders};
pub use api::config::{handle_save_configuration, handle_get_configuration, handle_update_server_password, handle_update_server_name};
pub use api::handlers::{handle_static_file, handle_ping, handle_connect_code};
pub use api::http::start_http_server;
pub use api::https::start_https_server;
pub use api::state::AppState;
pub use models::config::{Configuration, IncomingConfiguration, ServerPasswordUpdate, ServerNameUpdate, ConfigurationResponse};
pub use constants::{DEFAULT_HTTPS_PORT, DEFAULT_HTTP_PORT};
pub use utils::network::find_available_port;
pub use utils::token::generate_secure_token;

// Re-export error types for custom rejection handling
pub use api::config::{ConfigNotFoundError, ConfigGetError};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
