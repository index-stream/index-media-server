use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use serde_json::Value;

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

/// Profile model for database storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub created_at: i64, // Unix timestamp
}

impl Profile {
    /// Create a new profile instance
    pub fn new(name: String, color: String) -> Self {
        Self {
            id: 0, // Will be set by database
            name,
            color,
            created_at: Utc::now().timestamp(),
        }
    }
    
    /// Get the creation time as a DateTime
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(|| Utc::now())
    }
}

/// Index model for database storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Index {
    pub id: i64,
    pub name: String,
    pub r#type: String, // 'videos', 'photos', 'audio'
    pub is_plugin: i64, // 0 = false, 1 = true
    pub icon: Option<String>,
    pub created_at: i64, // Unix timestamp
    pub metadata: String, // JSON string
}

impl Index {
    /// Create a new index instance
    pub fn new(name: String, r#type: String, icon: Option<String>, metadata: Value) -> Self {
        Self {
            id: 0, // Will be set by database
            name,
            r#type,
            is_plugin: 0, // Default to false
            icon,
            created_at: Utc::now().timestamp(),
            metadata: serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".to_string()),
        }
    }
    
    /// Get the creation time as a DateTime
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(|| Utc::now())
    }
    
    /// Get metadata as parsed JSON
    pub fn metadata_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_str(&self.metadata)
    }
    
    /// Set metadata from JSON value
    pub fn set_metadata(&mut self, metadata: &Value) -> Result<(), serde_json::Error> {
        self.metadata = serde_json::to_string(metadata)?;
        Ok(())
    }
}

/// Scan job model for database storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScanJob {
    pub id: i64,
    pub index_id: i64,
    pub status: String, // 'queued', 'scanning'
    pub created_at: i64, // Unix timestamp
    pub updated_at: i64, // Unix timestamp
}

impl ScanJob {
    /// Create a new scan job instance
    pub fn new(index_id: i64, status: String) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: 0, // Will be set by database
            index_id,
            status,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Get the creation time as a DateTime
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(|| Utc::now())
    }
    
    /// Get the update time as a DateTime
    pub fn updated_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.updated_at, 0).unwrap_or_else(|| Utc::now())
    }
}

/// Video item model for database storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VideoItem {
    pub id: i64,
    pub index_id: i64,
    pub r#type: String, // 'video', 'movie', 'show', 'season', 'episode'
    pub parent_id: Option<i64>,
    pub title: String,
    pub sort_title: Option<String>,
    pub year: Option<i64>,
    pub number: Option<i64>, // season or episode number
    pub metadata: String, // JSON string
    pub added_at: i64, // Unix timestamp
    pub latest_added_at: i64, // Unix timestamp
    pub created_at: i64, // Unix timestamp
    pub updated_at: i64, // Unix timestamp
}

impl VideoItem {
    /// Create a new video item instance
    pub fn new(
        index_id: i64,
        r#type: String,
        title: String,
        parent_id: Option<i64>,
        metadata: Value,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: 0, // Will be set by database
            index_id,
            r#type,
            parent_id,
            title: title.clone(),
            sort_title: None,
            year: None,
            number: None,
            metadata: serde_json::to_string(&metadata).unwrap_or_else(|_| "{}".to_string()),
            added_at: now,
            latest_added_at: now,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Get the creation time as a DateTime
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(|| Utc::now())
    }
    
    /// Get the update time as a DateTime
    pub fn updated_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.updated_at, 0).unwrap_or_else(|| Utc::now())
    }
    
    /// Get metadata as parsed JSON
    pub fn metadata_json(&self) -> Result<Value, serde_json::Error> {
        serde_json::from_str(&self.metadata)
    }
    
    /// Set metadata from JSON value
    pub fn set_metadata(&mut self, metadata: &Value) -> Result<(), serde_json::Error> {
        self.metadata = serde_json::to_string(metadata)?;
        Ok(())
    }
}

/// Video version model for database storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VideoVersion {
    pub id: i64,
    pub item_id: i64,
    pub edition: Option<String>,
    pub source: Option<String>,
    pub container: Option<String>,
    pub resolution: Option<String>,
    pub hdr: i64, // 0 = false, 1 = true
    pub audio_channels: Option<i64>,
    pub bitrate: Option<i64>,
    pub runtime_ms: Option<i64>,
    pub probe_version: Option<String>,
    pub created_at: i64, // Unix timestamp
    pub updated_at: i64, // Unix timestamp
}

impl VideoVersion {
    /// Create a new video version instance
    pub fn new(item_id: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: 0, // Will be set by database
            item_id,
            edition: None,
            source: None,
            container: None,
            resolution: None,
            hdr: 0, // Default to false
            audio_channels: None,
            bitrate: None,
            runtime_ms: None,
            probe_version: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Get the creation time as a DateTime
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(|| Utc::now())
    }
    
    /// Get the update time as a DateTime
    pub fn updated_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.updated_at, 0).unwrap_or_else(|| Utc::now())
    }
}

/// Video part model for database storage
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VideoPart {
    pub id: i64,
    pub version_id: i64,
    pub path: String,
    pub size: Option<i64>,
    pub mtime: Option<i64>,
    pub part_index: i64,
    pub duration_ms: Option<i64>,
    pub fast_hash: Option<String>,
    pub created_at: i64, // Unix timestamp
    pub updated_at: i64, // Unix timestamp
}

impl VideoPart {
    /// Create a new video part instance
    pub fn new(version_id: i64, path: String, part_index: i64) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: 0, // Will be set by database
            version_id,
            path,
            size: None,
            mtime: None,
            part_index,
            duration_ms: None,
            fast_hash: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Get the creation time as a DateTime
    pub fn created_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created_at, 0).unwrap_or_else(|| Utc::now())
    }
    
    /// Get the update time as a DateTime
    pub fn updated_at_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.updated_at, 0).unwrap_or_else(|| Utc::now())
    }
}
