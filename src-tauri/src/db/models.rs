use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Token model for database storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Token {
    pub token: String,
    pub user_agent: String,
    pub created_at: i64, // Unix timestamp
}

impl Token {
    /// Create a new token instance
    pub fn new(token: String, user_agent: String) -> Self {
        Self {
            token,
            user_agent,
            created_at: Utc::now().timestamp(),
        }
    }
    
    /// Get the creation time as a DateTime
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(|| Utc::now())
    }
}
