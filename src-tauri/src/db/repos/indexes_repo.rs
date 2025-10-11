use sqlx::SqlitePool;
use anyhow::Result;
use crate::db::models::Index;
use serde_json::Value;

/// Repository for index database operations
#[derive(Debug)]
pub struct IndexesRepo {
    pool: SqlitePool,
}

impl IndexesRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// Add a new index to the database
    pub async fn add_index(&self, name: String, r#type: String, icon: Option<String>, metadata: Value) -> Result<i64> {
        let index = Index::new(name, r#type, icon, metadata);
        
        let result = sqlx::query(
            "INSERT INTO indexes (name, type, is_plugin, icon, created_at, metadata) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&index.name)
        .bind(&index.r#type)
        .bind(index.is_plugin)
        .bind(&index.icon)
        .bind(index.created_at)
        .bind(&index.metadata)
        .execute(&self.pool)
        .await?;
        
        Ok(result.last_insert_rowid())
    }
    
    /// Get all indexes
    pub async fn get_all_indexes(&self) -> Result<Vec<Index>> {
        let indexes = sqlx::query_as::<_, Index>("SELECT * FROM indexes ORDER BY created_at ASC")
            .fetch_all(&self.pool)
            .await?;
        
        Ok(indexes)
    }
    
    /// Get an index by ID
    pub async fn get_index_by_id(&self, id: i64) -> Result<Option<Index>> {
        let index = sqlx::query_as::<_, Index>("SELECT * FROM indexes WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(index)
    }
    
    /// Get indexes by type
    pub async fn get_indexes_by_type(&self, r#type: &str) -> Result<Vec<Index>> {
        let indexes = sqlx::query_as::<_, Index>("SELECT * FROM indexes WHERE type = ? ORDER BY created_at ASC")
            .bind(r#type)
            .fetch_all(&self.pool)
            .await?;
        
        Ok(indexes)
    }
    
    /// Update an index
    pub async fn update_index(&self, id: i64, name: String, icon: Option<String>, metadata: Value) -> Result<()> {
        let metadata_json = serde_json::to_string(&metadata)?;
        
        sqlx::query("UPDATE indexes SET name = ?, icon = ?, metadata = ? WHERE id = ?")
            .bind(&name)
            .bind(&icon)
            .bind(&metadata_json)
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Delete an index
    pub async fn delete_index(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM indexes WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Check if an index name already exists (excluding the given ID)
    pub async fn name_exists(&self, name: &str, exclude_id: Option<i64>) -> Result<bool> {
        let query = if exclude_id.is_some() {
            "SELECT COUNT(*) FROM indexes WHERE name = ? AND id != ?"
        } else {
            "SELECT COUNT(*) FROM indexes WHERE name = ?"
        };
        
        let result = if let Some(id) = exclude_id {
            sqlx::query_scalar::<_, i64>(query)
                .bind(name)
                .bind(id)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_scalar::<_, i64>(query)
                .bind(name)
                .fetch_one(&self.pool)
                .await?
        };
        
        Ok(result > 0)
    }
}
