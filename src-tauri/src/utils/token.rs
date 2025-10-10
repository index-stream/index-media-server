use rand::RngCore;
use base64::{Engine as _, engine::general_purpose};
use sha2::{Sha256, Digest};
use sqlx::SqlitePool;
use crate::db::repos::TokensRepo;

/// Token repository instance for database operations
static TOKEN_REPO: std::sync::OnceLock<TokensRepo> = std::sync::OnceLock::new();

/// Generate a cryptographically secure 256-bit random token in base64url format
pub fn generate_secure_token() -> String {
    let mut random_bytes = [0u8; 32]; // 256 bits = 32 bytes
    rand::thread_rng().fill_bytes(&mut random_bytes);
    general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
}

/// Initialize the token repository with a database pool
pub fn init_token_repo(pool: SqlitePool) {
    let repo = TokensRepo::new(pool);
    TOKEN_REPO.set(repo).expect("Failed to initialize token repository");
}

/// Add a new token to storage (stores the hashed token)
pub async fn add_token_to_storage(token: &str, user_agent: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repo = TOKEN_REPO.get().ok_or("Token repository not initialized")?;
    
    // Store the hashed token instead of the plain token
    let hashed_token = hash_token(token);
    repo.add_token(hashed_token, user_agent.to_string()).await?;
    Ok(())
}

/// Check if a token exists in storage (checks against hashed tokens)
pub async fn token_exists(token: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let repo = TOKEN_REPO.get().ok_or("Token repository not initialized")?;
    
    // Check against hashed token
    let hashed_token = hash_token(token);
    Ok(repo.token_exists(&hashed_token).await?)
}

/// Hash a token using SHA256
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    general_purpose::STANDARD.encode(hasher.finalize())
}
