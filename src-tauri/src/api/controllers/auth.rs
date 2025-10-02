use crate::api::router::{HttpRequest, HttpResponse, extract_user_agent};
use crate::models::config::Configuration;
use argon2::{Argon2, PasswordVerifier};
use argon2::password_hash::PasswordHashString;
use uuid::Uuid;
use serde_json;
use std::future::Future;
use std::pin::Pin;

/// Token storage structure
#[derive(serde::Serialize, serde::Deserialize)]
struct TokenInfo {
    user_agent: String,
    created_at: String,
}

/// Load configuration from file
async fn load_configuration() -> Result<Option<Configuration>, Box<dyn std::error::Error + Send + Sync>> {
    let config_path = std::env::current_dir()?.join("data").join("config.json");
    
    if !config_path.exists() {
        return Ok(None);
    }
    
    let config_json = std::fs::read_to_string(config_path)?;
    let config: Configuration = serde_json::from_str(&config_json)?;
    Ok(Some(config))
}

/// Verify password against stored hash
fn verify_password(password: &str, hash: &str) -> bool {
    if hash.is_empty() {
        // No password set, allow access
        return true;
    }
    
    if password.is_empty() {
        // Password required but not provided
        return false;
    }
    
    // Parse the stored hash
    let parsed_hash = match PasswordHashString::new(hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    
    // Verify the password
    let argon2 = Argon2::default();
    argon2.verify_password(password.as_bytes(), &parsed_hash.password_hash()).is_ok()
}

/// Get the data directory path for certificate storage
fn get_cert_data_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let mut data_dir = std::env::current_dir()?;
    data_dir.push("data");
    data_dir.push("certs");
    
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&data_dir)?;
    
    Ok(data_dir)
}

/// Get the full path for a certificate file
fn get_cert_file_path(filename: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let mut path = get_cert_data_dir()?;
    path.push(filename);
    Ok(path)
}

/// Load token storage from file
fn load_token_storage() -> Result<std::collections::HashMap<String, TokenInfo>, std::io::Error> {
    let token_file_path = get_cert_file_path("tokens.json").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))?;
    
    if !token_file_path.exists() {
        return Ok(std::collections::HashMap::new());
    }
    
    let content = std::fs::read_to_string(&token_file_path)?;
    let tokens: std::collections::HashMap<String, TokenInfo> = serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)))?;
    Ok(tokens)
}

/// Save token storage to file
fn save_token_storage(tokens: &std::collections::HashMap<String, TokenInfo>) -> Result<(), std::io::Error> {
    let token_file_path = get_cert_file_path("tokens.json").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))?;
    let content = serde_json::to_string_pretty(tokens)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)))?;
    std::fs::write(&token_file_path, content)?;
    Ok(())
}

/// Add a new token to storage
fn add_token_to_storage(token: &str, user_agent: &str) -> Result<(), std::io::Error> {
    let mut tokens = load_token_storage()?;
    
    let token_info = TokenInfo {
        user_agent: user_agent.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    tokens.insert(token.to_string(), token_info);
    save_token_storage(&tokens)?;
    Ok(())
}

/// Check if a token exists in storage
fn token_exists(token: &str) -> Result<bool, std::io::Error> {
    let tokens = load_token_storage()?;
    Ok(tokens.contains_key(token))
}

/// Handle login endpoint
pub fn handle_login(request: &HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> {
    let request = request.clone();
    Box::pin(async move {
        let user_agent = extract_user_agent(&request.headers);
        
        // Parse JSON body
        let body = request.body.as_ref().ok_or("No body provided")?;
        let login_data: serde_json::Value = serde_json::from_str(body)?;
        
        let password = login_data.get("password")
            .and_then(|p| p.as_str())
            .unwrap_or("");
        
        // Load configuration to check password
        match load_configuration().await? {
            Some(config) => {
                if verify_password(password, &config.password) {
                    // Generate auth token
                    let auth_token = Uuid::new_v4().to_string();
                    
                    // Store token with user agent
                    if let Err(e) = add_token_to_storage(&auth_token, &user_agent) {
                        eprintln!("Warning: Failed to store token: {}", e);
                    }
                    
                    let response_body = serde_json::json!({
                        "success": true,
                        "message": "Login successful",
                        "token": auth_token
                    });
                    
                    Ok(HttpResponse::new(200)
                        .with_cors()
                        .with_header("Set-Cookie", &format!("auth_token={}; HttpOnly; Secure; SameSite=Strict", auth_token))
                        .with_json_body(&response_body.to_string()))
                } else {
                    let response_body = serde_json::json!({
                        "success": false,
                        "message": "Invalid password"
                    });
                    
                    Ok(HttpResponse::new(401)
                        .with_cors()
                        .with_json_body(&response_body.to_string()))
                }
            }
            None => {
                // No configuration file, allow access without password
                let auth_token = Uuid::new_v4().to_string();
                
                // Store token with user agent
                if let Err(e) = add_token_to_storage(&auth_token, &user_agent) {
                    eprintln!("Warning: Failed to store token: {}", e);
                }
                
                let response_body = serde_json::json!({
                    "success": true,
                    "message": "Login successful (no password required)",
                    "token": auth_token
                });
                
                Ok(HttpResponse::new(200)
                    .with_cors()
                    .with_header("Set-Cookie", &format!("auth_token={}; HttpOnly; Secure; SameSite=Strict", auth_token))
                    .with_json_body(&response_body.to_string()))
            }
        }
    })
}

/// Handle token check endpoint
pub fn handle_token_check(request: &HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> {
    let request = request.clone();
    Box::pin(async move {
        // Extract token from query parameters
        let query_start = request.path.find('?');
        let token = if let Some(start) = query_start {
            let query_string = &request.path[start + 1..];
            if let Some(token_start) = query_string.find("token=") {
                let token_value = &query_string[token_start + 6..];
                // Remove any additional parameters after the token
                if let Some(ampersand) = token_value.find('&') {
                    &token_value[..ampersand]
                } else {
                    token_value
                }
            } else {
                ""
            }
        } else {
            ""
        };
        
        if token.is_empty() {
            let response_body = serde_json::json!({
                "error": "Missing token"
            });
            
            return Ok(HttpResponse::new(400)
                .with_cors()
                .with_json_body(&response_body.to_string()));
        }
        
        match token_exists(token) {
            Ok(exists) => {
                if exists {
                    let response_body = serde_json::json!({
                        "success": true,
                        "token": token,
                        "valid": true
                    });
                    
                    Ok(HttpResponse::new(200)
                        .with_cors()
                        .with_json_body(&response_body.to_string()))
                } else {
                    let response_body = serde_json::json!({
                        "error": "Token not found"
                    });
                    
                    Ok(HttpResponse::new(404)
                        .with_cors()
                        .with_json_body(&response_body.to_string()))
                }
            }
            Err(_) => {
                let response_body = serde_json::json!({
                    "error": "Server error"
                });
                
                Ok(HttpResponse::new(500)
                    .with_cors()
                    .with_json_body(&response_body.to_string()))
            }
        }
    })
}