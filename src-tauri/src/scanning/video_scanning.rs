use crate::api::state::AppState;
use crate::db::repos::{IndexesRepo, VideoRepo};
use crate::utils::hash::calculate_fast_hash;
use crate::utils::classifier::{classify_path, MediaType};
use std::path::Path;
use serde_json::Value;

/// Scan a single index (depth-first search for video files)
pub async fn scan_video_index(indexes_repo: &IndexesRepo, index: &crate::db::models::Index, app_state: &AppState) -> Result<(), anyhow::Error> {
    println!("🔍 Scanning index '{}' (ID: {})", index.name, index.id);
    
    // Parse metadata to get folders
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
    
    if folders.is_empty() {
        println!("⚠️  No folders configured for index '{}'", index.name);
        return Ok(());
    }
    
    println!("📁 Scanning {} folder(s) for video files...", folders.len());
    
    let pre_scan_timestamp = chrono::Utc::now().timestamp();
    let mut total_videos = 0;
    
    // Create video repository for database operations
    let video_repo = VideoRepo::new(app_state.db_pool.clone());
    
    // Process each folder
    for folder_path in folders {
        println!("📂 Scanning folder: {}", folder_path);
        
        match scan_folder_recursive(&folder_path, &video_repo, index.id).await {
            Ok(video_count) => {
                println!("✅ Found {} video(s) in folder: {}", video_count, folder_path);
                total_videos += video_count;
            }
            Err(e) => {
                eprintln!("❌ Error scanning folder '{}': {}", folder_path, e);
                // Continue with other folders even if one fails
            }
        }
    }
    
    println!("🎬 Total videos found: {}", total_videos);
    
    // Clean up deleted files from database
    println!("🧹 Cleaning up deleted files from database...");
    let cleanup_result = cleanup_deleted_files(&video_repo, index.id, pre_scan_timestamp).await;
    match cleanup_result {
        Ok((deleted_parts, deleted_versions, deleted_items)) => {
            println!("🗑️  Cleanup complete: {} parts, {} versions, {} items deleted", 
                     deleted_parts, deleted_versions, deleted_items);
        }
        Err(e) => {
            eprintln!("❌ Error during cleanup: {}", e);
            // Continue anyway - cleanup errors shouldn't stop the scan
        }
    }
    
    // Update status to done and set last_scanned_at to current time
    let now = chrono::Utc::now().timestamp();
    indexes_repo.update_scan_status_with_timestamp(index.id, "done".to_string(), Some(now)).await?;
    
    println!("✅ Completed scan for index '{}' (ID: {})", index.name, index.id);
    
    Ok(())
}

/// Recursively scan a folder for video files using depth-first search
async fn scan_folder_recursive(folder_path: &str, video_repo: &VideoRepo, index_id: i64) -> Result<usize, anyhow::Error> {
    let path = Path::new(folder_path);
    
    if !path.exists() {
        return Err(anyhow::anyhow!("Folder does not exist: {}", folder_path));
    }
    
    if !path.is_dir() {
        return Err(anyhow::anyhow!("Path is not a directory: {}", folder_path));
    }
    
    let mut video_count = 0;
    let mut dirs_to_process = vec![path.to_path_buf()];
    
    // Video file extensions to look for
    let video_extensions = [
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "ts", "m2ts", "webm", "mpeg", "mpg"
    ];
    
    // Depth-first search using a stack
    while let Some(current_dir) = dirs_to_process.pop() {
        let mut entries = tokio::fs::read_dir(&current_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                // Add subdirectory to stack for later processing
                dirs_to_process.push(entry_path);
            } else if entry_path.is_file() {
                // Check if it's a video file
                if let Some(extension) = entry_path.extension() {
                    if let Some(ext_str) = extension.to_str() {
                        let ext_lower = ext_str.to_lowercase();
                        if video_extensions.contains(&ext_lower.as_str()) {
                            // Classify the video file
                            let classified = classify_path(entry_path.to_string_lossy().as_ref());
                            println!("🎥 {} -> {:?}", 
                                entry_path.file_name().unwrap_or_default().to_string_lossy(),
                                classified.media_type
                            );
                            
                            match classified.media_type {
                                MediaType::Extra => {
                                    if let Some(extra) = classified.extra {
                                        println!("   🎬 Extra: {}", extra.path);
                                    }
                                }
                                MediaType::TvEpisode => {
                                    if let Some(tv) = classified.tv_episode {
                                        println!("   📺 TV: {} S{}E{} (year: {:?})", 
                                            tv.show_name,
                                            tv.season,
                                            tv.episode,
                                            tv.year.map_or("None".to_string(), |y| y.to_string())
                                        );
                                    }
                                }
                                MediaType::Movie => {
                                    if let Some(movie) = classified.movie {
                                        println!("   📽️  Movie: {} ({})", 
                                            movie.title, 
                                            movie.year.map_or("Unknown".to_string(), |y| y.to_string())
                                        );
                                    }
                                }
                                MediaType::Generic => {
                                    if let Some(generic) = classified.generic {
                                        println!("   📄 Generic: {}", generic.title);
                                    }
                                }
                            }
                            // Process the video file with database operations
                            match process_video_file(&entry_path, video_repo, index_id).await {
                                Ok(()) => {
                                    println!("🎥 {}", entry_path.display());
                                    video_count += 1;
                                }
                                Err(e) => {
                                    eprintln!("❌ Failed to process video file {}: {}", entry_path.display(), e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(video_count)
}

/// Process a single video file and update the database
async fn process_video_file(file_path: &Path, video_repo: &VideoRepo, index_id: i64) -> Result<(), anyhow::Error> {
    // Get file metadata
    let metadata = file_path.metadata()?;
    let file_size = metadata.len() as i64;
    let mtime = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
    
    // Calculate fast hash
    let fast_hash = calculate_fast_hash(file_path).await?;
    
    // Extract title from filename (without extension)
    let title = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Check if video_part exists with same size + fast_hash
    let existing_parts = video_repo.get_video_parts_by_size_and_hash(file_size, &fast_hash).await?;
    
    if let Some(existing_part) = existing_parts.first() {
        // Video part exists, check if path is the same
        if existing_part.path == file_path_str {
            // Same path, just update updated_at
            video_repo.update_video_part_updated_at(existing_part.id).await?;
        } else {
            // Different path, update path and updated_at
            video_repo.update_video_part_path(existing_part.id, file_path_str, mtime).await?;
        }
    } else {
        // Video part doesn't exist, need to create video_item and video_version first
        
        // Check if video_item exists with same title
        let existing_items = video_repo.get_video_items_by_title(index_id, &title).await?;
        
        let item_id = if let Some(existing_item) = existing_items.first() {
            // Video item exists, use the first one
            existing_item.id
        } else {
            // Create new video item
            video_repo.add_video_item(
                index_id,
                "video".to_string(),
                title.clone(),
                None, // parent_id
                Value::Object(serde_json::Map::new()) // empty metadata for now
            ).await?
        };
        
        // Create new video version (minimal details for now)
        let version_id = video_repo.add_video_version_with_params(
            item_id,
            None, // edition
            None, // source
            None, // container
            None, // resolution
            None, // hdr
            None, // audio_channels
            None, // bitrate
            None, // runtime_ms
            None  // probe_version
        ).await?;
        
        // Create new video part
        video_repo.add_video_part_with_params(
            version_id,
            file_path_str,
            Some(file_size),
            Some(mtime),
            0, // part_index
            None, // duration_ms
            Some(fast_hash)
        ).await?;
    }
    
    Ok(())
}

/// Clean up deleted files from the database
/// Returns (deleted_parts_count, deleted_versions_count, deleted_items_count)
async fn cleanup_deleted_files(
    video_repo: &VideoRepo, 
    index_id: i64, 
    pre_scan_timestamp: i64
) -> Result<(usize, usize, usize), anyhow::Error> {
    let mut deleted_parts = 0;
    let mut deleted_versions = 0;
    let mut deleted_items = 0;
    
    // Get all video items for this index
    let video_items = video_repo.get_video_items_by_index(index_id).await?;
    
    for video_item in video_items {
        // Get all video versions for this item
        let video_versions = video_repo.get_video_versions_by_item(video_item.id).await?;
        
        for video_version in video_versions {
            // Get all video parts for this version
            let video_parts = video_repo.get_video_parts_by_version(video_version.id).await?;
            
            // Check each video part
            for video_part in video_parts {
                if video_part.updated_at < pre_scan_timestamp {
                    // This part wasn't updated during scanning, so it was deleted
                    println!("🗑️  Deleting video part: {}", video_part.path);
                    video_repo.delete_video_part(video_part.id).await?;
                    deleted_parts += 1;
                }
            }
            
            // Check if this version now has no parts
            let remaining_parts = video_repo.get_video_parts_by_version(video_version.id).await?;
            if remaining_parts.is_empty() {
                println!("🗑️  Deleting empty video version: {}", video_version.id);
                video_repo.delete_video_version(video_version.id).await?;
                deleted_versions += 1;
            }
        }
        
        // Check if this item now has no versions
        let remaining_versions = video_repo.get_video_versions_by_item(video_item.id).await?;
        if remaining_versions.is_empty() {
            println!("🗑️  Deleting empty video item: {}", video_item.title);
            video_repo.delete_video_item(video_item.id).await?;
            deleted_items += 1;
        }
    }
    
    Ok((deleted_parts, deleted_versions, deleted_items))
}
