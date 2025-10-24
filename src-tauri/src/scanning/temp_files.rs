//! Temporary file management for video scanning
//! 
//! This module handles temporary files used during video scanning to store
//! new content and extras before processing them into the database.

use crate::utils::video_classifier::{MediaType, TvEpisodeInfo, MovieInfo, ExtraInfo, GenericInfo};

/// Temporary file data structures for new content
#[derive(Debug, Clone)]
pub struct TempVideoItem {
    pub file_path: String,
    pub media_type: MediaType,
    pub tv_episode: Option<TvEpisodeInfo>,
    pub movie: Option<MovieInfo>,
    pub generic: Option<GenericInfo>,
    pub file_size: i64,
    pub mtime: i64,
    pub fast_hash: String,
}

/// Temporary file data structures for extras
#[derive(Debug, Clone)]
pub struct TempExtraItem {
    pub file_path: String,
    pub extra: ExtraInfo,
    pub file_size: i64,
    pub mtime: i64,
    pub fast_hash: String,
}

/// Manager for temporary files during scanning
pub struct TempFileManager {
    temp_dir: std::path::PathBuf,
    new_content_items: Vec<TempVideoItem>,
    extra_items: Vec<TempExtraItem>,
}

impl TempFileManager {
    /// Create a new temporary file manager
    pub fn new(index_id: i64) -> Result<Self, anyhow::Error> {
        let temp_dir = std::env::temp_dir().join("index_media_server").join(format!("scan_{}", index_id));
        std::fs::create_dir_all(&temp_dir)?;
        
        Ok(Self {
            temp_dir,
            new_content_items: Vec::new(),
            extra_items: Vec::new(),
        })
    }
    
    /// Clean up any existing temporary files
    pub fn cleanup_existing_files(&self) -> Result<(), anyhow::Error> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
            std::fs::create_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }
    
    /// Add a new video item to temporary storage
    pub fn add_new_content(&mut self, item: TempVideoItem) -> Result<(), anyhow::Error> {
        self.new_content_items.push(item);
        Ok(())
    }
    
    /// Add an extra item to temporary storage
    pub fn add_extra(&mut self, item: TempExtraItem) -> Result<(), anyhow::Error> {
        self.extra_items.push(item);
        Ok(())
    }
    
    /// Load all new content items
    pub fn load_new_content(&self) -> Result<Vec<TempVideoItem>, anyhow::Error> {
        Ok(self.new_content_items.clone())
    }
    
    /// Load all extra items
    pub fn load_extras(&self) -> Result<Vec<TempExtraItem>, anyhow::Error> {
        Ok(self.extra_items.clone())
    }
    
    /// Clear all temporary items after processing
    pub fn clear_items(&mut self) {
        self.new_content_items.clear();
        self.extra_items.clear();
    }
    
    /// Clean up all temporary files
    pub fn cleanup(&self) -> Result<(), anyhow::Error> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }
    
    /// Get the temporary directory path
    pub fn temp_dir(&self) -> &std::path::Path {
        &self.temp_dir
    }
}

/// Source path tracker for validation
pub struct SourcePathTracker {
    source_path: Option<String>,
}

impl SourcePathTracker {
    pub fn new() -> Self {
        Self {
            source_path: None,
        }
    }
    
    /// Track a source path and validate it doesn't conflict
    pub fn track_source_path(&mut self, source_path: &str, _file_path: &str) -> Result<(), anyhow::Error> {
        match &self.source_path {
            None => {
                // No source path set yet, set it
                self.source_path = Some(source_path.to_string());
                Ok(())
            }
            Some(existing_path) => {
                if existing_path == source_path {
                    // Same source path, OK
                    Ok(())
                } else {
                    // Different source path, error
                    Err(anyhow::anyhow!(
                        "Source path conflict: expected '{}' but found '{}'",
                        source_path, existing_path
                    ))
                }
            }
        }
    }
    
    /// Check if a source path has been tracked
    pub fn has_source_path(&self, source_path: &str) -> bool {
        self.source_path.as_ref().map_or(false, |path| path == source_path)
    }
    
    /// Get the tracked source path
    pub fn get_source_path(&self) -> Option<&String> {
        self.source_path.as_ref()
    }
    
    /// Remove a source path from tracking (when finished processing a folder)
    /// Returns true if a source path was actually removed, false if none was set
    pub fn remove_source_path(&mut self, source_path: &str) -> bool {
        if self.source_path.as_ref().map_or(false, |path| path == source_path) {
            self.source_path = None;
            true
        } else {
            //only print if source_path exists
            if self.source_path.is_some() {
                println!("❌ Expected source path {} but found {}", source_path, self.source_path.as_ref().unwrap());
            }
            false
        }
    }
}
