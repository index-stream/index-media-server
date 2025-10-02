use super::router::{RouteHandler, HttpRequest, HttpResponse};

/// Get content type based on file extension
fn get_content_type(path: &str) -> &'static str {
    match path.split('.').last().unwrap_or("") {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

/// Serve static file
async fn serve_static_file(path: &str) -> Result<(Vec<u8>, &'static str), Box<dyn std::error::Error + Send + Sync>> {
    // Get the current directory and go up one level to find the web directory
    let current_dir = std::env::current_dir()?;
    let project_root = current_dir.parent().unwrap_or(&current_dir);
    let web_dir = project_root.join("web");
    
    // Strip query parameters from the path
    let clean_path = if let Some(query_start) = path.find('?') {
        &path[..query_start]
    } else {
        path
    };
    
    let file_path = if clean_path == "/" {
        web_dir.join("index.html")
    } else {
        web_dir.join(clean_path.trim_start_matches('/'))
    };
    
    // Check if file exists
    if !file_path.exists() || !file_path.is_file() {
        return Err("File not found".into());
    }
    
    // Read file content
    let content = std::fs::read(&file_path)?;
    let content_type = get_content_type(file_path.to_str().unwrap_or(""));
    
    Ok((content, content_type))
}

/// Static file controller for serving web assets
pub struct StaticController;

impl RouteHandler for StaticController {
    async fn handle(&self, request: &HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        // Only handle GET requests for non-API paths
        if request.method == "GET" && !request.path.starts_with("/api/") {
            self.serve_file(&request.path).await
        } else {
            Err("Route not handled".into())
        }
    }
}

impl StaticController {
    pub fn new() -> Self {
        Self
    }

    async fn serve_file(&self, path: &str) -> Result<HttpResponse, Box<dyn std::error::Error>> {
        match serve_static_file(path).await {
            Ok((content, content_type)) => {
                // Convert content to string for response
                let content_str = String::from_utf8_lossy(&content);
                
                Ok(HttpResponse::new(200)
                    .with_cors()
                    .with_header("Content-Type", content_type)
                    .with_body(&content_str))
            }
            Err(_) => {
                // File not found, return 404
                Ok(HttpResponse::new(404)
                    .with_cors()
                    .with_header("Content-Type", "text/html")
                    .with_body("Not Found"))
            }
        }
    }
}
