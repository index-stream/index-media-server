use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::server::TlsStream;
use tokio::net::TcpStream;
use std::future::Future;
use std::pin::Pin;

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
}

impl HttpResponse {
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: Vec::new(),
            body: None,
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
            _ => "HTTP/1.1 200 OK",
        };

        let mut response = format!("{}\r\n", status_line);
        
        // Add headers
        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }
        
        // Add content length
        let body_len = self.body.as_ref().map_or(0, |b| b.len());
        response.push_str(&format!("Content-Length: {}\r\n", body_len));
        response.push_str("\r\n");
        
        // Send headers
        stream.write_all(response.as_bytes()).await?;
        
        // Send body if present
        if let Some(body) = self.body {
            stream.write_all(body.as_bytes()).await?;
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
        for route in &self.routes {
            if route.method == request.method && self.matches_path(&route.path_pattern, &request.path) {
                return (route.handler)(request).await;
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
        
        false
    }
}

/// Handle a single HTTPS connection using the router
pub async fn handle_connection_with_router(
    mut tls_stream: TlsStream<TcpStream>,
    router: &Router,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut buffer = [0; 2048];
    let n = tls_stream.read(&mut buffer).await?;
    
    if n == 0 {
        return Ok(());
    }
    
    let request_str = String::from_utf8_lossy(&buffer[..n]);
    
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
