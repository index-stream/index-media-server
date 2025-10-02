use rand::RngCore;
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Token storage structure
#[derive(Serialize, Deserialize, Clone)]
pub struct TokenInfo {
    pub user_agent: String,
    pub created_at: String,
}

/// Global token cache to avoid frequent file reads
type TokenCache = Arc<Mutex<Option<HashMap<String, TokenInfo>>>>;

/// Global token cache instance
static TOKEN_CACHE: std::sync::OnceLock<TokenCache> = std::sync::OnceLock::new();

/// Generate a cryptographically secure 256-bit random token in base64url format
pub fn generate_secure_token() -> String {
    let mut random_bytes = [0u8; 32]; // 256 bits = 32 bytes
    rand::thread_rng().fill_bytes(&mut random_bytes);
    general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
}

/// Hash a token using SHA256
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    general_purpose::STANDARD.encode(hasher.finalize())
}

/// Get tokens from cache if available, otherwise return None
fn get_token_cache() -> Option<HashMap<String, TokenInfo>> {
    let cache = TOKEN_CACHE.get_or_init(|| Arc::new(Mutex::new(None)));
    let cache_guard = cache.lock().unwrap();
    cache_guard.clone()
}

/// Update the token cache with new data
fn update_token_cache(tokens: HashMap<String, TokenInfo>) {
    let cache = TOKEN_CACHE.get_or_init(|| Arc::new(Mutex::new(None)));
    let mut cache_guard = cache.lock().unwrap();
    *cache_guard = Some(tokens);
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
fn load_token_storage() -> Result<HashMap<String, TokenInfo>, std::io::Error> {
    let token_file_path = get_cert_file_path("tokens.json").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))?;
    
    if !token_file_path.exists() {
        return Ok(HashMap::new());
    }
    
    let content = std::fs::read_to_string(&token_file_path)?;
    let tokens: HashMap<String, TokenInfo> = serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)))?;
    Ok(tokens)
}

/// Save token storage to file and update cache
fn save_token_storage(tokens: &HashMap<String, TokenInfo>) -> Result<(), std::io::Error> {
    let token_file_path = get_cert_file_path("tokens.json").map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{}", e)))?;
    let content = serde_json::to_string_pretty(tokens)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{}", e)))?;
    std::fs::write(&token_file_path, content)?;
    
    // Update cache after successful disk write
    update_token_cache(tokens.clone());
    
    Ok(())
}

/// Add a new token to storage (stores the hashed token)
pub fn add_token_to_storage(token: &str, user_agent: &str) -> Result<(), std::io::Error> {
    let mut tokens = load_token_storage()?;
    
    let token_info = TokenInfo {
        user_agent: user_agent.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    
    // Store the hashed token instead of the plain token
    let hashed_token = hash_token(token);
    tokens.insert(hashed_token, token_info);
    save_token_storage(&tokens)?;
    Ok(())
}

/// Check if a token exists in storage (checks against hashed tokens, uses cache)
pub fn token_exists(token: &str) -> Result<bool, std::io::Error> {
    let hashed_token = hash_token(token);
    
    // Check if cache exists and contains the token
    if let Some(cached_tokens) = get_token_cache() {
        if cached_tokens.contains_key(&hashed_token) {
            return Ok(true); // Token found in cache, return early
        }
    }
    
    // Cache doesn't exist or doesn't contain the token, load from disk
    let tokens = load_token_storage()?;
    update_token_cache(tokens.clone());
    
    Ok(tokens.contains_key(&hashed_token))
}
