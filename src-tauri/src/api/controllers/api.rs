use crate::api::router::{HttpRequest, HttpResponse};
use crate::models::config::Configuration;
use serde_json;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::OnceLock;

// Cache for serverId to avoid repeated file reads
static SERVER_ID_CACHE: OnceLock<Arc<Mutex<Option<String>>>> = OnceLock::new();

/// Load configuration from file (same as in auth.rs)
async fn load_configuration() -> Result<Option<Configuration>, Box<dyn std::error::Error + Send + Sync>> {
    let config_path = std::env::current_dir()?.join("data").join("config.json");
    
    if !config_path.exists() {
        return Ok(None);
    }
    
    let config_json = std::fs::read_to_string(config_path)?;
    let config: Configuration = serde_json::from_str(&config_json)?;
    Ok(Some(config))
}

/// Get cached serverId or load it from configuration
async fn get_server_id() -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    let cache = SERVER_ID_CACHE.get_or_init(|| Arc::new(Mutex::new(None)));
    
    // Check if we already have a cached value
    {
        let cached_id = cache.lock().unwrap();
        if cached_id.is_some() {
            return Ok(cached_id.clone());
        }
    }
    
    // Load configuration and cache the serverId
    if let Some(config) = load_configuration().await? {
        let mut cached_id = cache.lock().unwrap();
        *cached_id = Some(config.id.clone());
        Ok(Some(config.id))
    } else {
        Ok(None)
    }
}

/// Handle ping endpoint
pub fn handle_ping(_request: &HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> {
    Box::pin(async move {
        // Get serverId from cache or configuration
        let server_id = get_server_id().await.ok().flatten();
        
        let mut response_data = serde_json::json!({
            "status": "ok",
            "message": "Index Media Server is running",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        // Add serverId to response if available
        if let Some(id) = server_id {
            response_data["serverId"] = serde_json::Value::String(id);
        }
        
        Ok(HttpResponse::new(200)
            .with_cors()
            .with_json_body(&response_data.to_string()))
    })
}