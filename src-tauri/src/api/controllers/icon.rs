use crate::api::router::{HttpRequest, HttpResponse};
use std::fs;
use std::path::PathBuf;

/// Handle icon endpoint for serving custom icons by index ID
pub fn handle_index_icon(request: &HttpRequest) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> {
    let request = request.clone();
    Box::pin(async move {
        // Extract index_id from the path
        // Expected path format: /api/index/{index_id}/icon
        let path_parts: Vec<&str> = request.path.split('/').collect();
        
        if path_parts.len() < 5 || path_parts[1] != "api" || path_parts[2] != "index" || path_parts[4] != "icon" {
            return Ok(HttpResponse::new(404)
                .with_cors()
                .with_body("Not Found"));
        }
        
        let index_id = path_parts[3];
        if index_id.is_empty() {
            return Ok(HttpResponse::new(400)
                .with_cors()
                .with_body("Bad Request: Invalid index ID"));
        }
        
        // Get the data directory path
        let data_dir = match std::env::current_dir() {
            Ok(dir) => dir.join("data").join("icons"),
            Err(_) => {
                return Ok(HttpResponse::new(500)
                    .with_cors()
                    .with_body("Internal Server Error"));
            }
        };
        
        // Try to find the icon file with common image extensions
        let extensions = ["png", "jpg", "jpeg", "gif", "webp", "svg"];
        let mut icon_path: Option<PathBuf> = None;
        
        for ext in &extensions {
            let test_path = data_dir.join(format!("{}.{}", index_id, ext));
            if test_path.exists() {
                icon_path = Some(test_path);
                break;
            }
        }
        
        match icon_path {
            Some(path) => {
                // Read the icon file
                match fs::read(&path) {
                    Ok(icon_data) => {
                        // Determine content type based on file extension
                        let content_type = match path.extension().and_then(|ext| ext.to_str()) {
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("gif") => "image/gif",
                            Some("webp") => "image/webp",
                            Some("svg") => "image/svg+xml",
                            _ => "application/octet-stream",
                        };
                        
                        Ok(HttpResponse::new(200)
                            .with_header("Content-Type", content_type)
                            .with_header("Cache-Control", "public, max-age=31536000") // Cache for 1 year
                            .with_cors()
                            .with_binary_body(icon_data))
                    }
                    Err(_) => {
                        Ok(HttpResponse::new(500)
                            .with_cors()
                            .with_body("Internal Server Error"))
                    }
                }
            }
            None => {
                Ok(HttpResponse::new(404)
                    .with_cors()
                    .with_body("Icon not found"))
            }
        }
    })
}
