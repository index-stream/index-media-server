use serde::Serialize;
use crate::db::models::{Profile as DbProfile, Index as DbIndex};

/// Database-based configuration response that fetches profiles and indexes from database
#[derive(Debug, Serialize)]
pub struct DatabaseConfigurationResponse {
    pub id: String,
    pub name: String,
    pub profiles: Vec<ProfileResponse>,
    pub indexes: Vec<IndexResponse>,
}

/// Profile response structure matching the old API format
#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub id: String,
    pub name: String,
    pub color: String,
}

impl From<DbProfile> for ProfileResponse {
    fn from(profile: DbProfile) -> Self {
        Self {
            id: profile.id.to_string(),
            name: profile.name,
            color: profile.color,
        }
    }
}

/// Index response structure matching the old API format
#[derive(Debug, Serialize)]
pub struct IndexResponse {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub icon: String,
    pub folders: Vec<String>,
}

impl From<DbIndex> for IndexResponse {
    fn from(index: DbIndex) -> Self {
        // Parse metadata to extract folders
        let folders = if let Ok(meta) = index.metadata_json() {
            if let Some(folders_array) = meta.get("folders") {
                if let Some(arr) = folders_array.as_array() {
                    arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        
        Self {
            id: index.id.to_string(),
            name: index.name,
            r#type: index.r#type,
            icon: index.icon.unwrap_or_else(|| "custom".to_string()),
            folders,
        }
    }
}

/// Filtered index response for auth endpoints (only id, name, type, icon)
#[derive(Debug, Serialize)]
pub struct FilteredIndexResponse {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub icon: String,
}

impl From<DbIndex> for FilteredIndexResponse {
    fn from(index: DbIndex) -> Self {
        Self {
            id: index.id.to_string(),
            name: index.name,
            r#type: index.r#type,
            icon: index.icon.unwrap_or_else(|| "custom".to_string()),
        }
    }
}
