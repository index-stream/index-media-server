use sqlx::SqlitePool;
use anyhow::Result;
use crate::db::models::{VideoItem, VideoVersion, VideoPart};
use serde_json::Value;

/// Repository for video-related database operations
#[derive(Debug)]
pub struct VideoRepo {
    pool: SqlitePool,
}

impl VideoRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    // Video Items
    
    /// Add a new video item
    pub async fn add_video_item(&self, index_id: i64, r#type: String, title: String, parent_id: Option<i64>, metadata: Value) -> Result<i64> {
        let video_item = VideoItem::new(index_id, r#type, title, parent_id, metadata);
        
        let result = sqlx::query(
            "INSERT INTO video_items (index_id, type, parent_id, title, sort_title, year, number, metadata, added_at, latest_added_at, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(video_item.index_id)
        .bind(&video_item.r#type)
        .bind(&video_item.parent_id)
        .bind(&video_item.title)
        .bind(&video_item.sort_title)
        .bind(&video_item.year)
        .bind(&video_item.number)
        .bind(&video_item.metadata)
        .bind(video_item.added_at)
        .bind(video_item.latest_added_at)
        .bind(video_item.created_at)
        .bind(video_item.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(result.last_insert_rowid())
    }
    
    /// Get video items by index
    pub async fn get_video_items_by_index(&self, index_id: i64) -> Result<Vec<VideoItem>> {
        let video_items = sqlx::query_as::<_, VideoItem>(
            "SELECT * FROM video_items WHERE index_id = ? ORDER BY latest_added_at DESC"
        )
        .bind(index_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(video_items)
    }
    
    /// Get video items by type
    pub async fn get_video_items_by_type(&self, index_id: i64, r#type: &str) -> Result<Vec<VideoItem>> {
        let video_items = sqlx::query_as::<_, VideoItem>(
            "SELECT * FROM video_items WHERE index_id = ? AND type = ? ORDER BY latest_added_at DESC"
        )
        .bind(index_id)
        .bind(r#type)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(video_items)
    }
    
    /// Get video item by ID
    pub async fn get_video_item_by_id(&self, id: i64) -> Result<Option<VideoItem>> {
        let video_item = sqlx::query_as::<_, VideoItem>("SELECT * FROM video_items WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(video_item)
    }
    
    /// Get children of a video item
    pub async fn get_video_item_children(&self, parent_id: i64) -> Result<Vec<VideoItem>> {
        let video_items = sqlx::query_as::<_, VideoItem>(
            "SELECT * FROM video_items WHERE parent_id = ? ORDER BY number ASC, title ASC"
        )
        .bind(parent_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(video_items)
    }
    
    // Video Versions
    
    /// Add a new video version
    pub async fn add_video_version(&self, item_id: i64) -> Result<i64> {
        let video_version = VideoVersion::new(item_id);
        
        let result = sqlx::query(
            "INSERT INTO video_versions (item_id, edition, source, container, resolution, hdr, audio_channels, bitrate, runtime_ms, probe_version, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(video_version.item_id)
        .bind(&video_version.edition)
        .bind(&video_version.source)
        .bind(&video_version.container)
        .bind(&video_version.resolution)
        .bind(video_version.hdr)
        .bind(&video_version.audio_channels)
        .bind(&video_version.bitrate)
        .bind(&video_version.runtime_ms)
        .bind(&video_version.probe_version)
        .bind(video_version.created_at)
        .bind(video_version.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(result.last_insert_rowid())
    }
    
    /// Get video versions by item
    pub async fn get_video_versions_by_item(&self, item_id: i64) -> Result<Vec<VideoVersion>> {
        let video_versions = sqlx::query_as::<_, VideoVersion>(
            "SELECT * FROM video_versions WHERE item_id = ? ORDER BY created_at ASC"
        )
        .bind(item_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(video_versions)
    }
    
    // Video Parts
    
    /// Add a new video part
    pub async fn add_video_part(&self, version_id: i64, path: String, part_index: i64) -> Result<i64> {
        let video_part = VideoPart::new(version_id, path, part_index);
        
        let result = sqlx::query(
            "INSERT INTO video_parts (version_id, path, size, mtime, part_index, duration_ms, fast_hash, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(video_part.version_id)
        .bind(&video_part.path)
        .bind(&video_part.size)
        .bind(&video_part.mtime)
        .bind(video_part.part_index)
        .bind(&video_part.duration_ms)
        .bind(&video_part.fast_hash)
        .bind(video_part.created_at)
        .bind(video_part.updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(result.last_insert_rowid())
    }
    
    /// Get video parts by version
    pub async fn get_video_parts_by_version(&self, version_id: i64) -> Result<Vec<VideoPart>> {
        let video_parts = sqlx::query_as::<_, VideoPart>(
            "SELECT * FROM video_parts WHERE version_id = ? ORDER BY part_index ASC"
        )
        .bind(version_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(video_parts)
    }
    
    /// Get video part by path
    pub async fn get_video_part_by_path(&self, path: &str) -> Result<Option<VideoPart>> {
        let video_part = sqlx::query_as::<_, VideoPart>("SELECT * FROM video_parts WHERE path = ?")
            .bind(path)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(video_part)
    }
}
