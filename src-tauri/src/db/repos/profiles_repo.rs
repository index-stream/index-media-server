use sqlx::SqlitePool;
use anyhow::Result;
use crate::db::models::Profile;

/// Repository for profile database operations
#[derive(Debug)]
pub struct ProfilesRepo {
    pool: SqlitePool,
}

impl ProfilesRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
    
    /// Add a new profile to the database
    pub async fn add_profile(&self, name: String, color: String) -> Result<i64> {
        let profile = Profile::new(name, color);
        
        let result = sqlx::query(
            "INSERT INTO profiles (name, color, created_at) VALUES (?, ?, ?)"
        )
        .bind(&profile.name)
        .bind(&profile.color)
        .bind(profile.created_at)
        .execute(&self.pool)
        .await?;
        
        Ok(result.last_insert_rowid())
    }
    
    /// Get all profiles
    pub async fn get_all_profiles(&self) -> Result<Vec<Profile>> {
        let profiles = sqlx::query_as::<_, Profile>("SELECT * FROM profiles ORDER BY created_at ASC")
            .fetch_all(&self.pool)
            .await?;
        
        Ok(profiles)
    }
    
    /// Get a profile by ID
    pub async fn get_profile_by_id(&self, id: i64) -> Result<Option<Profile>> {
        let profile = sqlx::query_as::<_, Profile>("SELECT * FROM profiles WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        
        Ok(profile)
    }
    
    /// Update a profile
    pub async fn update_profile(&self, id: i64, name: String, color: String) -> Result<()> {
        sqlx::query("UPDATE profiles SET name = ?, color = ? WHERE id = ?")
            .bind(&name)
            .bind(&color)
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    /// Delete a profile
    pub async fn delete_profile(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM profiles WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
}
