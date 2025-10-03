use crate::api::router::{HttpRequest, HttpResponse, extract_user_agent};
use crate::models::config::Configuration;
use crate::utils::token::{generate_secure_token, add_token_to_storage, token_exists};
use argon2::{Argon2, PasswordVerifier};
use argon2::password_hash::PasswordHashString;
use serde_json;
use std::future::Future;
use std::pin::Pin;

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

/// Handle login endpoint
pub fn handle_login(request: &HttpRequest) -> Pin<Box<dyn Future<Output = Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static>> {
    let request = request.clone();
    Box::pin(async move {
        let user_agent = extract_user_agent(&request.headers);
        
        // Parse JSON body
        let body = match request.body.as_ref() {
            Some(body) => body,
            None => {
                let response_body = serde_json::json!({
                    "success": false,
                    "message": "No request body provided"
                });
                
                return Ok(HttpResponse::new(400)
                    .with_cors()
                    .with_json_body(&response_body.to_string()));
            }
        };
        let login_data: serde_json::Value = match serde_json::from_str(body) {
            Ok(data) => data,
            Err(_) => {
                let response_body = serde_json::json!({
                    "success": false,
                    "message": "Invalid JSON in request body"
                });
                
                return Ok(HttpResponse::new(400)
                    .with_cors()
                    .with_json_body(&response_body.to_string()));
            }
        };
        
        let password = login_data.get("password")
            .and_then(|p| p.as_str())
            .unwrap_or("");
        
        // Load configuration to check password (guaranteed to exist due to router check)
        let config = load_configuration().await?.ok_or("Configuration not found")?;
        
        if verify_password(password, &config.password) {
            // Generate cryptographically secure auth token
            let auth_token = generate_secure_token();
            
            // Store token with user agent
            if let Err(e) = add_token_to_storage(&auth_token, &user_agent) {
                eprintln!("Warning: Failed to store token: {}", e);
            }
            
            let response_body = serde_json::json!({
                "success": true,
                "message": "Login successful",
                "token": auth_token,
                "serverId": config.id,
                "serverName": config.name,
                "profiles": config.profiles
            });
            
            Ok(HttpResponse::new(200)
                .with_cors()
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
        
        // Load configuration (guaranteed to exist due to router check)
        let config = load_configuration().await?.ok_or("Configuration not found")?;
        
        // Check token validity
        match token_exists(token) {
            Ok(exists) => {
                if exists {
                    let response_body = serde_json::json!({
                        "success": true,
                        "token": token,
                        "valid": true,
                        "serverId": config.id,
                        "serverName": config.name,
                        "profiles": config.profiles
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