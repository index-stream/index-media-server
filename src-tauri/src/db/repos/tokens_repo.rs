use sqlx::SqlitePool;
use anyhow::Result;
use crate::db::models::Token;

/// Repository for token database operations
#[derive(Debug)]
pub struct TokensRepo {
    pool: SqlitePool,
}

impl TokensRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// Add a new token to the database
    pub async fn add_token(&self, token: String, user_agent: String) -> Result<()> {
        let token_model = Token::new(token, user_agent);
        
        sqlx::query(
            "INSERT INTO tokens (token, user_agent, created_at) VALUES (?, ?, ?)"
        )
        .bind(&token_model.token)
        .bind(&token_model.user_agent)
        .bind(token_model.created_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    /// Check if a token exists in the database
    pub async fn token_exists(&self, token: &str) -> Result<bool> {
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM tokens WHERE token = ?"
        )
        .bind(token)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result > 0)
    }
    
    /// Get all tokens (for debugging/admin purposes)
    pub async fn get_all_tokens(&self) -> Result<Vec<Token>> {
        let tokens = sqlx::query_as::<_, Token>("SELECT * FROM tokens ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        
        Ok(tokens)
    }
    
    /// Delete a specific token
    pub async fn delete_token(&self, token: &str) -> Result<()> {
        sqlx::query("DELETE FROM tokens WHERE token = ?")
            .bind(token)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Delete tokens older than the specified timestamp
    pub async fn delete_old_tokens(&self, older_than: i64) -> Result<u64> {
        let result = sqlx::query("DELETE FROM tokens WHERE created_at < ?")
            .bind(older_than)
            .execute(&self.pool)
            .await?;
        
        Ok(result.rows_affected())
    }
}
