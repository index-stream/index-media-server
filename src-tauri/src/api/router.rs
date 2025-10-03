use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::server::TlsStream;
use tokio::net::TcpStream;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::OnceLock;
use serde_json;

/// HTTP request information
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<String>,
    pub body: Option<String>,
}

/// HTTP response builder
pub struct HttpResponse {
    status_code: u16,
    headers: Vec<(String, String)>,
    body: Option<String>,
    binary_body: Option<Vec<u8>>,
}

impl HttpResponse {
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: Vec::new(),
            body: None,
            binary_body: None,
        }
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn with_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    pub fn with_json_body(mut self, body: &str) -> Self {
        self.headers.push(("Content-Type".to_string(), "application/json".to_string()));
        self.body = Some(body.to_string());
        self
    }

    pub fn with_binary_body(mut self, body: Vec<u8>) -> Self {
        self.binary_body = Some(body);
        self
    }

    pub fn with_cors(mut self) -> Self {
        self.headers.push(("Access-Control-Allow-Origin".to_string(), "*".to_string()));
        self.headers.push(("Access-Control-Allow-Methods".to_string(), "GET, POST, OPTIONS".to_string()));
        self.headers.push(("Access-Control-Allow-Headers".to_string(), "content-type".to_string()));
        self
    }

    pub async fn send(self, stream: &mut TlsStream<TcpStream>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status_line = match self.status_code {
            200 => "HTTP/1.1 200 OK",
            400 => "HTTP/1.1 400 Bad Request",
            401 => "HTTP/1.1 401 Unauthorized",
            404 => "HTTP/1.1 404 Not Found",
            500 => "HTTP/1.1 500 Internal Server Error",
            503 => "HTTP/1.1 503 Service Unavailable",
            _ => "HTTP/1.1 200 OK",
        };

        let mut response = format!("{}\r\n", status_line);
        
        // Add headers
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        // Add content length
        let body_len = self.body.as_ref().map_or(0, |b| b.len()) + self.binary_body.as_ref().map_or(0, |b| b.len());
        response.push_str(&format!("Content-Length: {}\r\n", body_len));
        response.push_str("\r\n");
        
        // Send headers
        stream.write_all(response.as_bytes()).await?;
        
        // Send body if present
        if let Some(body) = self.body {
            stream.write_all(body.as_bytes()).await?;
        }
        
        // Send binary body if present
        if let Some(binary_body) = self.binary_body {
            stream.write_all(&binary_body).await?;
        }
        
        stream.flush().await?;
        Ok(())
    }
}

/// Parse HTTP request from raw bytes
pub fn parse_http_request(request: &str) -> Option<HttpRequest> {
    let lines: Vec<&str> = request.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let request_line = lines[0];
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();
    
    // Find headers
    let mut headers = Vec::new();
    let mut body_start = None;
    
    for (i, line) in lines.iter().enumerate() {
        if line.is_empty() {
            body_start = Some(i + 1);
            break;
        }
        if i > 0 { // Skip the request line
            headers.push(line.to_string());
        }
    }
    
    // Extract body if present
    let body = if let Some(start) = body_start {
        if start < lines.len() {
            Some(lines[start..].join("\n"))
        } else {
            None
        }
    } else {
        None
    };

    Some(HttpRequest {
        method,
        path,
        headers,
        body,
    })
}

/// Extract user agent from headers
pub fn extract_user_agent(headers: &[String]) -> String {
    headers.iter()
        .find(|line| line.to_lowercase().starts_with("user-agent:"))
        .and_then(|line| line.splitn(2, ':').nth(1))
        .map(|ua| ua.trim().to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

// Cache for server initialization status to avoid repeated file reads
static SERVER_INIT_CACHE: OnceLock<Arc<Mutex<Option<bool>>>> = OnceLock::new();

/// Check if server configuration exists (with caching)
async fn check_server_initialized() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let cache = SERVER_INIT_CACHE.get_or_init(|| Arc::new(Mutex::new(None)));
    
    // Check if we already have a cached value (only cache true values)
    {
        let cached_status = cache.lock().unwrap();
        if let Some(true) = *cached_status {
            return Ok(true);
        }
    }
    
    // Check file system
    let config_path = std::env::current_dir()?.join("data").join("config.json");
    let exists = config_path.exists();
    
    // Only cache if the server is initialized (true), never cache false
    if exists {
        let mut cached_status = cache.lock().unwrap();
        *cached_status = Some(true);
    }
    
    Ok(exists)
}

/// Route handler function type
pub type RouteHandler = fn(&HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>>;

/// Route definition
#[derive(Clone)]
pub struct Route {
    pub method: String,
    pub path_pattern: String,
    pub handler: RouteHandler,
}

/// Main router using standard Rust patterns
#[derive(Clone)]
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
        }
    }

    pub fn add_route(&mut self, method: &str, path_pattern: &str, handler: RouteHandler) {
        self.routes.push(Route {
            method: method.to_string(),
            path_pattern: path_pattern.to_string(),
            handler,
        });
    }

    pub async fn handle_request(&self, request: &HttpRequest) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Check if server is initialized before processing any request
        if !check_server_initialized().await? {
            let response_body = serde_json::json!({
                "success": false,
                "error": "Server not initialized",
                "message": "Please configure the server before attempting to connect"
            });
            
            return Ok(HttpResponse::new(503)
                .with_cors()
                .with_json_body(&response_body.to_string()));
        }
        
        for route in &self.routes {
            if route.method == request.method && self.matches_path(&route.path_pattern, &request.path) {
                return match (route.handler)(request).await {
                    Ok(response) => Ok(response),
                    Err(e) => {
                        eprintln!("Handler error: {}", e);
                        let response_body = serde_json::json!({
                            "success": false,
                            "error": "Internal server error",
                            "message": "An unexpected error occurred"
                        });
                        
                        Ok(HttpResponse::new(500)
                            .with_cors()
                            .with_json_body(&response_body.to_string()))
                    }
                };
            }
        }
        
        // No route matched, return 404
        Ok(HttpResponse::new(404)
            .with_cors()
            .with_body("Not Found"))
    }

    fn matches_path(&self, pattern: &str, path: &str) -> bool {
        if pattern == path {
            return true;
        }
        
        // Handle prefix matching for patterns ending with *
        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len() - 1];
            return path.starts_with(prefix);
        }
        
        // Handle path parameter patterns like /api/index/{index_id}/icon
        if pattern.contains("{") && pattern.contains("}") {
            return self.matches_path_with_params(pattern, path);
        }
        
        false
    }
    
    fn matches_path_with_params(&self, pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();
        
        if pattern_parts.len() != path_parts.len() {
            return false;
        }
        
        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
                // This is a parameter, any non-empty value matches
                if path_part.is_empty() {
                    return false;
                }
            } else if pattern_part != path_part {
                // Exact match required for non-parameter parts
                return false;
            }
        }
        
        true
    }
}

/// Read a complete HTTP request from the TLS stream
async fn read_complete_http_request(
    tls_stream: &mut TlsStream<TcpStream>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0; 1024];
    
    loop {
        let n = tls_stream.read(&mut temp_buffer).await?;
        if n == 0 {
            break;
        }
        
        buffer.extend_from_slice(&temp_buffer[..n]);
        
        // Check if we have a complete HTTP request
        let request_str = String::from_utf8_lossy(&buffer);
        
        // Look for the end of headers (double CRLF)
        if let Some(header_end) = request_str.find("\r\n\r\n") {
            let headers = &request_str[..header_end + 4];
            
            // Check if there's a Content-Length header
            if let Some(content_length_start) = headers.find("Content-Length:") {
                if let Some(content_length_end) = headers[content_length_start..].find("\r\n") {
                    let content_length_str = &headers[content_length_start + 15..content_length_start + content_length_end];
                    if let Ok(content_length) = content_length_str.trim().parse::<usize>() {
                        let body_start = header_end + 4;
                        let expected_total = body_start + content_length;
                        
                        if buffer.len() >= expected_total {
                            // We have the complete request
                            return Ok(String::from_utf8_lossy(&buffer[..expected_total]).to_string());
                        }
                        // Continue reading to get the complete body
                    }
                }
            } else {
                // No Content-Length header, assume request is complete after headers
                return Ok(request_str.to_string());
            }
        }
        
        // Prevent infinite loops with very large requests
        if buffer.len() > 1024 * 1024 * 100 { // 100MB limit
            return Err("Request too large".into());
        }
    }
    
    if buffer.is_empty() {
        Err("Empty request".into())
    } else {
        Ok(String::from_utf8_lossy(&buffer).to_string())
    }
}

/// Handle a single HTTPS connection using the router
pub async fn handle_connection_with_router(
    mut tls_stream: TlsStream<TcpStream>,
    router: &Router,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    // Read the complete HTTP request
    let request_str = match read_complete_http_request(&mut tls_stream).await {
        Ok(req) => req,
        Err(_e) => return Ok(()),
    };
    
    // Parse HTTP request
    let request = match parse_http_request(&request_str) {
        Some(req) => req,
        None => return Ok(()),
    };
    
    // Handle CORS preflight
    if request.method == "OPTIONS" {
        let response = HttpResponse::new(200)
            .with_cors()
            .with_body("");
        response.send(&mut tls_stream).await?;
        return Ok(());
    }
    
    // Route the request
    let response = router.handle_request(&request).await?;
    response.send(&mut tls_stream).await?;
    
    Ok(())
}
