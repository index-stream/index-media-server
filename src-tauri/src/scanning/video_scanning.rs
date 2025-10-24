use crate::api::state::AppState;
use crate::db::repos::{IndexesRepo, VideoRepo};
use crate::utils::hash::calculate_fast_hash;
use crate::utils::video_classifier::{classify_path, MediaType, classify_movie_extra, classify_show_extra, MovieExtra, ShowExtra, GenericInfo};
use crate::scanning::{TempFileManager, SourcePathTracker, TempVideoItem, TempExtraItem};
use std::path::{Path, PathBuf};
use serde_json::Value;

/// Scan a single index (depth-first search for video files)
pub async fn scan_video_index(indexes_repo: &IndexesRepo, index: &crate::db::models::Index, app_state: &AppState) -> Result<(), anyhow::Error> {
    println!("🔍 Scanning index '{}' (ID: {})", index.name, index.id);
    
    // Initialize temporary file manager and cleanup any existing files
    let mut temp_manager = TempFileManager::new(index.id)?;
    temp_manager.cleanup_existing_files()?;
    println!("🧹 Cleaned up any existing temporary files from previous scans");
    
    // Initialize source path tracker
    let mut source_tracker = SourcePathTracker::new();
    
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
        
        match scan_folder_recursive(&folder_path, &video_repo, index.id, &mut temp_manager, &mut source_tracker).await {
            Ok(video_count) => {
                println!("✅ Found {} video(s) in folder: {}", video_count, folder_path);
                total_videos += video_count;
            }
            Err(e) => {
                eprintln!("❌ Error scanning folder '{}': {}", folder_path, e);
                // Continue with other folders even if one fails
            }
        }
        
        // Remove the folder we just processed from source path tracking
        // This prevents false conflicts when processing subsequent folders
        source_tracker.remove_source_path(&folder_path);
    }
    
    println!("🎬 Total videos found: {}", total_videos);
    
    // Process any remaining temporary files (for content without source paths)
    println!("📝 Processing remaining temporary files...");
    process_temp_files(&mut temp_manager, &video_repo, index.id, "").await?;
    
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
    
    // Clean up temporary files
    temp_manager.cleanup()?;
    println!("🧹 Cleaned up temporary files");
    
    // Update status to done and set last_scanned_at to current time
    let now = chrono::Utc::now().timestamp();
    indexes_repo.update_scan_status_with_timestamp(index.id, "done".to_string(), Some(now)).await?;
    
    println!("✅ Completed scan for index '{}' (ID: {})", index.name, index.id);
    
    Ok(())
}

/// Recursively scan a folder for video files using depth-first search
async fn scan_folder_recursive(
    folder_path: &str, 
    video_repo: &VideoRepo, 
    index_id: i64,
    temp_manager: &mut TempFileManager,
    source_tracker: &mut SourcePathTracker
) -> Result<usize, anyhow::Error> {
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
    
    // Process files in folder order before going deeper (breadth-first for files, then depth-first for dirs)
    while let Some(current_dir) = dirs_to_process.pop() {
        if current_dir.to_string_lossy().starts_with("REMOVE_FROM_TRACKER:") {
            let path_str = current_dir.to_string_lossy();
            let path_to_remove = path_str.strip_prefix("REMOVE_FROM_TRACKER:").unwrap();
            
            // If we successfully removed a source path, process temporary files
            if source_tracker.remove_source_path(path_to_remove) {
                println!("📝 Processing temporary files for completed source path: {}", path_to_remove);
                process_temp_files(temp_manager, video_repo, index_id, path_to_remove).await?;
                println!("✅ Temporary files processed for completed source path: {}", path_to_remove);
            }
            continue;
        }
        let mut entries = tokio::fs::read_dir(&current_dir).await?;
        let mut subdirs = Vec::new();
        
        // First pass: collect all entries and separate files from directories
        let mut file_entries = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            
            if entry_path.is_dir() {
                subdirs.push(entry_path);
            } else if entry_path.is_file() {
                file_entries.push(entry_path);
            }
        }
        
        // Process all files in the current directory first
        for entry_path in file_entries {
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
                        // Process the video file with temporary file system
                        match process_video_file(&entry_path, video_repo, index_id, temp_manager, source_tracker).await {
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
        
        // Push removal signal FIRST so it gets processed LAST (LIFO stack)
        // This ensures we remove the source path after processing all subdirectories
        let removal_signal = format!("REMOVE_FROM_TRACKER:{}", current_dir.to_string_lossy());
        dirs_to_process.push(PathBuf::from(removal_signal));
        
        // Then add subdirectories to stack for later processing
        for subdir in subdirs {
            dirs_to_process.push(subdir);
        }
    }
    
    Ok(video_count)
}

/// Process a single video file and either update database or add to temporary files
async fn process_video_file(
    file_path: &Path, 
    video_repo: &VideoRepo, 
    index_id: i64,
    temp_manager: &mut TempFileManager,
    source_tracker: &mut SourcePathTracker
) -> Result<(), anyhow::Error> {
    // Get file metadata
    let metadata = file_path.metadata()?;
    let file_size = metadata.len() as i64;
    let mtime = metadata.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs() as i64;
    
    // Calculate fast hash
    let fast_hash = calculate_fast_hash(file_path).await?;
    
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Check if video_part exists with same size + fast_hash
    let existing_parts = video_repo.get_video_parts_by_size_and_hash(file_size, &fast_hash).await?;
    
    if let Some(existing_part) = existing_parts.first() {
        // Video part exists, check if path is the same
        if existing_part.path == file_path_str {
            // Same path, just update updated_at
            video_repo.update_video_part_updated_at(existing_part.id).await?;
        } else {
            // Different path - check if this is a source path change that requires migration
            let classified = classify_path(&file_path_str);
            
            // Get the video item to check its current source path
            let video_version = video_repo.get_video_version_by_id(existing_part.version_id).await?
                .ok_or_else(|| anyhow::anyhow!("Video version not found"))?;
            let video_item = video_repo.get_video_item_by_id(video_version.item_id).await?
                .ok_or_else(|| anyhow::anyhow!("Video item not found"))?;
            
            // Determine new source path based on classification
            let new_source_path = match classified.media_type {
                MediaType::Movie => {
                    if let Some(movie) = &classified.movie {
                        let file_dir = file_path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str());
                        if let Some(folder_name) = file_dir {
                            if is_movie_in_matching_folder(&movie.title, movie.year, folder_name) {
                                file_path.parent().and_then(|p| p.to_str()).map(|s| s.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                MediaType::TvEpisode => {
                    if let Some(tv) = &classified.tv_episode {
                        Some(tv.source_path.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            };
            
            // Check if source path has changed and handle migration if needed
            if let Some(new_source_path) = new_source_path {
                if video_item.source_path.as_ref() != Some(&new_source_path) {
                    // Source path has changed, handle migration
                    if let Some(old_source_path) = &video_item.source_path {
                        handle_episode_migration(video_repo, existing_part.id, old_source_path, &new_source_path).await?;
                    } else {
                        // No old source path, just update the item
                        video_repo.update_video_item_source_path(video_item.id, Some(new_source_path)).await?;
                    }
                }
            }
            
            // Update path and updated_at
            video_repo.update_video_part_path(existing_part.id, file_path_str, mtime).await?;
        }
        return Ok(());
    }
    
    // Video part doesn't exist, classify the file
    let classified = classify_path(&file_path_str);
    
    // Handle extras separately
    if classified.media_type == MediaType::Extra {
        if let Some(extra) = classified.extra {
            let temp_extra = TempExtraItem {
                file_path: file_path_str,
                extra,
                file_size,
                mtime,
                fast_hash,
            };
            temp_manager.add_extra(temp_extra)?;
        }
        return Ok(());
    }
    
    // Handle movies, TV episodes, and generic content
    let source_path = match classified.media_type {
        MediaType::Movie => {
            if let Some(movie) = &classified.movie {
                // Check if movie is in a matching folder
                let file_dir = file_path.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str());
                if let Some(folder_name) = file_dir {
                    if is_movie_in_matching_folder(&movie.title, movie.year, folder_name) {
                        // Movie is in its own folder, use parent folder as source_path
                        file_path.parent().and_then(|p| p.to_str()).map(|s| s.to_string())
                    } else {
                        // Movie is not in its own folder, no source_path
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
        MediaType::TvEpisode => {
            if let Some(tv) = &classified.tv_episode {
                Some(tv.source_path.clone())
            } else {
                None
            }
        }
        MediaType::Generic => {
            // Generic content doesn't have a source path
            None
        }
        MediaType::Extra => {
            // Already handled above
            unreachable!()
        }
    };
    
    // Track source path for validation
    if let Some(source_path) = &source_path {
        source_tracker.track_source_path(source_path, &file_path_str)?;
    }
    
    // For movies without source_path, add immediately if not in a source_path structure
    if classified.media_type == MediaType::Movie && source_path.is_none() {
        // Check if we're already within a source_path structure
        if source_tracker.get_source_path().is_none() {
            // Add movie immediately
            add_movie_immediately(file_path, video_repo, index_id, classified, file_size, mtime, fast_hash).await?;
        } else {
            return Err(anyhow::anyhow!("Movie without source_path found within source_path structure"));
        }
        return Ok(());
    }
    
    // Add to temporary files for later processing
    let temp_item = TempVideoItem {
        file_path: file_path_str,
        media_type: classified.media_type,
        tv_episode: classified.tv_episode,
        movie: classified.movie,
        generic: classified.generic,
        file_size,
        mtime,
        fast_hash,
    };
    temp_manager.add_new_content(temp_item)?;
    
    Ok(())
}

/// Check if a movie matches its folder name (ignoring case, spaces, and dots)
fn is_movie_in_matching_folder(movie_title: &str, movie_year: Option<i32>, folder_name: &str) -> bool {
    // Normalize strings by removing spaces, dots, and converting to lowercase
    let normalize = |s: &str| s.replace(' ', "").replace('.', "").to_lowercase();
    
    let normalized_title = normalize(movie_title);
    let normalized_folder = normalize(folder_name);
    
    // Check if folder contains the movie title
    if !normalized_folder.contains(&normalized_title) {
        return false;
    }
    
    // If movie has a year, check if folder contains it
    if let Some(year) = movie_year {
        let year_str = year.to_string();
        if !normalized_folder.contains(&year_str) {
            return false;
        }
    }
    
    true
}

/// Add a movie immediately to the database (for movies without source_path)
async fn add_movie_immediately(
    file_path: &Path,
    video_repo: &VideoRepo,
    index_id: i64,
    _classified: crate::utils::video_classifier::ClassificationResult,
    file_size: i64,
    mtime: i64,
    fast_hash: String,
) -> Result<(), anyhow::Error> {
    let file_path_str = file_path.to_string_lossy().to_string();
    
    // Extract title from filename (without extension)
    let title = file_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    
    // Check if video_item exists with same title
    let existing_items = video_repo.get_video_items_by_title(index_id, &title).await?;
    
    let item_id = if let Some(existing_item) = existing_items.first() {
        // Video item exists, use the first one
        existing_item.id
    } else {
        // Create new video item
        video_repo.add_video_item(
            index_id,
            "movie".to_string(),
            title.clone(),
            None, // parent_id
            None, // source_path
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
    
    Ok(())
}

/// Process temporary files after scanning is complete
async fn process_temp_files(
    temp_manager: &mut TempFileManager,
    video_repo: &VideoRepo,
    index_id: i64,
    path_to_remove: &str,
) -> Result<(), anyhow::Error> {
    // Process new content first
    let new_content = temp_manager.load_new_content()?;
    println!("📝 Processing {} new content items...", new_content.len());
    
    for item in new_content {
        process_temp_video_item(item, video_repo, index_id).await?;
    }
    
    // Process extras after new content
    let extras = temp_manager.load_extras()?;
    println!("📝 Processing {} extra items...", extras.len());
    
    for item in extras {
        process_temp_extra_item(item, video_repo, index_id, path_to_remove).await?;
    }

    // Clear the temporary items after processing
    temp_manager.clear_items();
    println!("🧹 Cleared temporary items from memory");
    
    Ok(())
}

/// Process a temporary video item and add it to the database
async fn process_temp_video_item(
    item: TempVideoItem,
    video_repo: &VideoRepo,
    index_id: i64,
) -> Result<(), anyhow::Error> {
    match item.media_type {
        MediaType::Movie => {
            if let Some(ref movie) = item.movie {
                process_temp_movie(&item, movie.clone(), video_repo, index_id).await?;
            }
        }
        MediaType::TvEpisode => {
            if let Some(ref tv) = item.tv_episode {
                process_temp_tv_episode(&item, tv.clone(), video_repo, index_id).await?;
            }
        }
        MediaType::Generic => {
            if let Some(ref generic) = item.generic {
                process_temp_generic(&item, generic.clone(), video_repo, index_id).await?;
            }
        }
        MediaType::Extra => {
            // Extras are handled separately
            unreachable!()
        }
    }
    Ok(())
}

/// Process a temporary movie item
async fn process_temp_movie(
    item: &TempVideoItem,
    movie: crate::utils::video_classifier::MovieInfo,
    video_repo: &VideoRepo,
    index_id: i64,
) -> Result<(), anyhow::Error> {
    // Check if video_item exists with same source_path
    let existing_items = video_repo.get_video_items_by_source_path(index_id, &movie.source_path).await?;
    
    let item_id = if let Some(existing_item) = existing_items.first() {
        // Video item exists, use it
        existing_item.id
    } else {
        // Create new video item
        video_repo.add_video_item(
            index_id,
            "movie".to_string(),
            movie.title.clone(),
            None, // parent_id
            Some(movie.source_path.clone()),
            Value::Object(serde_json::Map::new()) // empty metadata for now
        ).await?
    };
    
    // Create new video version
    let version_id = video_repo.add_video_version_with_params(
        item_id,
        movie.version,
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
        item.file_path.clone(),
        Some(item.file_size),
        Some(item.mtime),
        0, // part_index
        None, // duration_ms
        Some(item.fast_hash.clone())
    ).await?;
    
    Ok(())
}

/// Process a temporary TV episode item
async fn process_temp_tv_episode(
    item: &TempVideoItem,
    tv: crate::utils::video_classifier::TvEpisodeInfo,
    video_repo: &VideoRepo,
    index_id: i64,
) -> Result<(), anyhow::Error> {
    // Step 1: Find or create the show (video_item with source_path)
    let existing_shows = video_repo.get_video_items_by_source_path(index_id, &tv.source_path).await?;
    
    let show_id = if let Some(existing_show) = existing_shows.first() {
        // Show exists, use it
        existing_show.id
    } else {
        // Create new show
        video_repo.add_video_item(
            index_id,
            "show".to_string(),
            tv.show_name.clone(),
            None, // parent_id
            Some(tv.source_path.clone()),
            Value::Object(serde_json::Map::new()) // empty metadata for now
        ).await?
    };
    
    // Step 2: Find or create the season (video_item child of show)
    let season_title = if tv.season == 0 {
        "Specials".to_string()
    } else {
        format!("Season {}", tv.season)
    };
    
    let existing_seasons = video_repo.get_video_items_by_parent_and_number(show_id, tv.season).await?;
    
    let season_id = if let Some(existing_season) = existing_seasons.first() {
        // Season exists, use it
        existing_season.id
    } else {
        // Create new season
        video_repo.add_video_item_with_number(
            index_id,
            "season".to_string(),
            season_title.clone(),
            Some(show_id), // parent_id
            None, // source_path (seasons don't have their own source path)
            Some(tv.season), // number
            Value::Object(serde_json::Map::new()) // empty metadata for now
        ).await?
    };
    
    // Step 3: Find or create the episode (video_item child of season)
    let episode_title = if let Some(title) = &tv.title {
        title.clone()
    } else if let Some(air_date) = &tv.air_date {
        // For air_date episodes, use the air_date as the title
        air_date.clone()
    } else {
        format!("Episode {}", tv.episode)
    };
    
    let existing_episodes = video_repo.get_video_items_by_parent_and_number(season_id, tv.episode).await?;
    
    let episode_id = if let Some(existing_episode) = existing_episodes.first() {
        // Episode exists, use it
        existing_episode.id
    } else {
        // Create new episode with metadata including air_date
        let mut metadata = serde_json::Map::new();
        if let Some(air_date) = &tv.air_date {
            metadata.insert("air_date".to_string(), Value::String(air_date.clone()));
        }
        
        video_repo.add_video_item_with_number(
            index_id,
            "episode".to_string(),
            episode_title.clone(),
            Some(season_id), // parent_id
            None, // source_path (episodes don't have their own source path)
            Some(tv.episode), // number
            Value::Object(metadata)
        ).await?
    };
    
    // Step 4: Create new video version linked to the episode
    let version_id = video_repo.add_video_version_with_params(
        episode_id,
        tv.version,
        None, // source
        None, // container
        None, // resolution
        None, // hdr
        None, // audio_channels
        None, // bitrate
        None, // runtime_ms
        None  // probe_version
    ).await?;
    
    // Step 5: Create new video part
    video_repo.add_video_part_with_params(
        version_id,
        item.file_path.clone(),
        Some(item.file_size),
        Some(item.mtime),
        0, // part_index
        None, // duration_ms
        Some(item.fast_hash.clone())
    ).await?;
    
    Ok(())
}

/// Process a temporary generic item
async fn process_temp_generic(
    item: &TempVideoItem,
    generic: crate::utils::video_classifier::GenericInfo,
    video_repo: &VideoRepo,
    index_id: i64,
) -> Result<(), anyhow::Error> {
    // Check if video_item exists with same title
    let existing_items = video_repo.get_video_items_by_title(index_id, &generic.title).await?;
    
    let item_id = if let Some(existing_item) = existing_items.first() {
        // Video item exists, use it
        existing_item.id
    } else {
        // Create new video item
        video_repo.add_video_item(
            index_id,
            "video".to_string(),
            generic.title.clone(),
            None, // parent_id
            None, // source_path
            Value::Object(serde_json::Map::new()) // empty metadata for now
        ).await?
    };
    
    // Create new video version
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
        item.file_path.clone(),
        Some(item.file_size),
        Some(item.mtime),
        0, // part_index
        None, // duration_ms
        Some(item.fast_hash.clone())
    ).await?;
    
    Ok(())
}

/// Process a temporary extra item and add it to the database
async fn process_temp_extra_item(
    item: TempExtraItem,
    video_repo: &VideoRepo,
    index_id: i64,
    path_to_remove: &str,
) -> Result<(), anyhow::Error> {
    // Check if the source_path is fully contained within the extra path
    if path_to_remove.is_empty() || !item.extra.path.contains(path_to_remove) {
        println!("🔍 Treating extra as generic video: {}", item.extra.path);
        // Treat as generic video instead
        let filename = std::path::Path::new(&item.extra.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let generic_info = GenericInfo {
            title: filename,
        };
        
        process_temp_generic_from_extra(item, generic_info, video_repo, index_id).await?;
        return Ok(());
    }
    
    println!("🔍 Checking if there's a video_item associated with the source_path: {}", path_to_remove);
    // Check if there's a video_item associated with the source_path
    let existing_items = video_repo.get_video_items_by_source_path(index_id, path_to_remove).await?;
    
    if existing_items.is_empty() {
        println!("🔍 No video_item found, treating extra as generic video: {}", item.extra.path);
        // No video_item found, treat as generic
        let filename = std::path::Path::new(&item.extra.path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let generic_info = GenericInfo {
            title: filename,
        };
        
        process_temp_generic_from_extra(item, generic_info, video_repo, index_id).await?;
        return Ok(());
    }
    
    let parent_item = &existing_items[0];
    
    // Determine if this is a movie or show extra
    match parent_item.r#type.as_str() {
        "movie" => {
            if let Some(movie_extra) = classify_movie_extra(&item.extra, path_to_remove) {
                process_temp_movie_extra(item, movie_extra, parent_item.id, video_repo, index_id).await?;
            }
        }
        "show" => {
            if let Some(show_extra) = classify_show_extra(&item.extra, path_to_remove) {
                process_temp_show_extra(item, show_extra, parent_item.id, video_repo, index_id).await?;
            }
        }
        _ => {
            println!("🔍 Unknown type, treating extra as generic video: {}", item.extra.path);
            // Unknown type, treat as generic
            let filename = std::path::Path::new(&item.extra.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            let generic_info = GenericInfo {
                title: filename,
            };
            
            process_temp_generic_from_extra(item, generic_info, video_repo, index_id).await?;
        }
    }
    
    Ok(())
}

/// Process a movie extra
async fn process_temp_movie_extra(
    item: TempExtraItem,
    movie_extra: MovieExtra,
    parent_item_id: i64,
    video_repo: &VideoRepo,
    index_id: i64,
) -> Result<(), anyhow::Error> {
    // Create extra video item with movie as parent
    let mut metadata = serde_json::Map::new();
    metadata.insert("extra_type".to_string(), Value::String(movie_extra.extra_type));
    
    let extra_item_id = video_repo.add_video_item(
        index_id,
        "extra".to_string(),
        movie_extra.title,
        Some(parent_item_id), // parent_id
        None, // source_path
        Value::Object(metadata)
    ).await?;
    
    // Create video version
    let version_id = video_repo.add_video_version_with_params(
        extra_item_id,
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
    
    // Create video part
    video_repo.add_video_part_with_params(
        version_id,
        item.file_path,
        Some(item.file_size),
        Some(item.mtime),
        0, // part_index
        None, // duration_ms
        Some(item.fast_hash)
    ).await?;
    
    Ok(())
}

/// Process a show extra
async fn process_temp_show_extra(
    item: TempExtraItem,
    show_extra: ShowExtra,
    parent_item_id: i64,
    video_repo: &VideoRepo,
    index_id: i64,
) -> Result<(), anyhow::Error> {
    let mut actual_parent_id = parent_item_id;
    
    // If this is for a specific season, find the season
    if let Some(season) = show_extra.season {
        let existing_seasons = video_repo.get_video_items_by_parent_and_number(parent_item_id, season).await?;
        if let Some(season_item) = existing_seasons.first() {
            actual_parent_id = season_item.id;
            
            // If this is for a specific episode, find the episode
            if let Some(episode) = show_extra.episode {
                let existing_episodes = video_repo.get_video_items_by_parent_and_number(actual_parent_id, episode).await?;
                if let Some(episode_item) = existing_episodes.first() {
                    actual_parent_id = episode_item.id;
                }
            }
        }
    }
    
    // Create extra video item with appropriate parent
    let mut metadata = serde_json::Map::new();
    metadata.insert("extra_type".to_string(), Value::String(show_extra.extra_type));
    
    let extra_item_id = video_repo.add_video_item(
        index_id,
        "extra".to_string(),
        show_extra.title,
        Some(actual_parent_id), // parent_id
        None, // source_path
        Value::Object(metadata)
    ).await?;
    
    // Create video version
    let version_id = video_repo.add_video_version_with_params(
        extra_item_id,
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
    
    // Create video part
    video_repo.add_video_part_with_params(
        version_id,
        item.file_path,
        Some(item.file_size),
        Some(item.mtime),
        0, // part_index
        None, // duration_ms
        Some(item.fast_hash)
    ).await?;
    
    Ok(())
}

/// Process a generic item from an extra
async fn process_temp_generic_from_extra(
    item: TempExtraItem,
    generic: GenericInfo,
    video_repo: &VideoRepo,
    index_id: i64,
) -> Result<(), anyhow::Error> {
    // Create generic video item
    let generic_item_id = video_repo.add_video_item(
        index_id,
        "video".to_string(),
        generic.title,
        None, // parent_id
        None, // source_path
        Value::Object(serde_json::Map::new()) // empty metadata for now
    ).await?;
    
    // Create video version
    let version_id = video_repo.add_video_version_with_params(
        generic_item_id,
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
    
    // Create video part
    video_repo.add_video_part_with_params(
        version_id,
        item.file_path,
        Some(item.file_size),
        Some(item.mtime),
        0, // part_index
        None, // duration_ms
        Some(item.fast_hash)
    ).await?;
    
    Ok(())
}

/// Handle episode migration when source path changes
/// This implements the 4 situations for moving episodes between shows
async fn handle_episode_migration(
    video_repo: &VideoRepo,
    video_part_id: i64,
    old_source_path: &str,
    new_source_path: &str,
) -> Result<(), anyhow::Error> {
    // Get the video part and its version/item
    let video_part = video_repo.get_video_part_by_id(video_part_id).await?
        .ok_or_else(|| anyhow::anyhow!("Video part not found"))?;
    
    let video_version = video_repo.get_video_version_by_id(video_part.version_id).await?
        .ok_or_else(|| anyhow::anyhow!("Video version not found"))?;
    
    let video_item = video_repo.get_video_item_by_id(video_version.item_id).await?
        .ok_or_else(|| anyhow::anyhow!("Video item not found"))?;
    
    // Check if old source path still exists
    let old_path_exists = std::path::Path::new(old_source_path).exists();
    
    // Check if new source path has a video item
    let new_path_items = video_repo.get_video_items_by_source_path(video_item.index_id, new_source_path).await?;
    let new_path_has_item = !new_path_items.is_empty();
    
    match (old_path_exists, new_path_has_item) {
        (false, false) => {
            // Old path doesn't exist, new path doesn't have video_item
            // Migrate old_path record to new path
            video_repo.update_video_item_source_path(video_item.id, Some(new_source_path.to_string())).await?;
        }
        (false, true) => {
            // Old path doesn't exist, new path already has a video_item
            // Move video_part and possibly video_version to new video_item
            let new_item = &new_path_items[0];
            move_video_part_to_item(video_repo, video_part_id, video_version.id, video_item.id, new_item.id).await?;
        }
        (true, false) => {
            // Old path still exists, new path doesn't have video_item
            // Create new video_item and move video_part and possible video_version to new video_item
            let new_item_id = video_repo.add_video_item(
                video_item.index_id,
                video_item.r#type.clone(),
                video_item.title.clone(),
                video_item.parent_id,
                Some(new_source_path.to_string()),
                serde_json::from_str(&video_item.metadata).unwrap_or(Value::Object(serde_json::Map::new()))
            ).await?;
            
            move_video_part_to_item(video_repo, video_part_id, video_version.id, video_item.id, new_item_id).await?;
        }
        (true, true) => {
            // Old path still exists, new path already has a video_item
            // Move video_part and possibly video_version to new video_item
            let new_item = &new_path_items[0];
            move_video_part_to_item(video_repo, video_part_id, video_version.id, video_item.id, new_item.id).await?;
        }
    }
    
    Ok(())
}

/// Move a video part and its version to a new video item
async fn move_video_part_to_item(
    video_repo: &VideoRepo,
    video_part_id: i64,
    video_version_id: i64,
    old_item_id: i64,
    new_item_id: i64,
) -> Result<(), anyhow::Error> {
    // Check if the version has other parts
    let other_parts = video_repo.get_video_parts_by_version(video_version_id).await?
        .into_iter()
        .filter(|part| part.id != video_part_id)
        .collect::<Vec<_>>();
    
    if other_parts.is_empty() {
        // This is the only part in the version, move the entire version
        video_repo.update_video_version_item_id(video_version_id, new_item_id).await?;
    } else {
        // Create a new version for this part in the new item
        let video_version = video_repo.get_video_version_by_id(video_version_id).await?
            .ok_or_else(|| anyhow::anyhow!("Video version not found"))?;
        
        let new_version_id = video_repo.add_video_version_with_params(
            new_item_id,
            video_version.edition,
            video_version.source,
            video_version.container,
            video_version.resolution,
            Some(video_version.hdr),
            video_version.audio_channels,
            video_version.bitrate,
            video_version.runtime_ms,
            video_version.probe_version
        ).await?;
        
        // Update the video part to use the new version
        video_repo.update_video_part_version_id(video_part_id, new_version_id).await?;
    }
    
    // Check if the old item now has no versions
    let remaining_versions = video_repo.get_video_versions_by_item(old_item_id).await?;
    if remaining_versions.is_empty() {
        video_repo.delete_video_item(old_item_id).await?;
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
        
        // Check if this item now has no versions or video_items with parent_id equal to this item's id
        let remaining_versions = video_repo.get_video_versions_by_item(video_item.id).await?;
        let remaining_children = video_repo.get_video_items_by_parent(video_item.id).await?;
        if remaining_versions.is_empty() && remaining_children.is_empty() {
            println!("🗑️  Deleting empty video item: {}", video_item.title);
            video_repo.delete_video_item(video_item.id).await?;
            deleted_items += 1;
        }
    }
    
    Ok((deleted_parts, deleted_versions, deleted_items))
}
