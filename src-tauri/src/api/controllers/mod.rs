pub mod auth;
pub mod static_files;
pub mod api;

pub use auth::{handle_login, handle_token_check};
pub use static_files::handle_static_files;
pub use api::handle_ping;
