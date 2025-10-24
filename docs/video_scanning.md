# Video Scanning Algorithm Documentation

## Overview

The video scanning algorithm is a sophisticated system designed to efficiently scan, classify, and manage media files in complex folder structures. It handles movies, TV shows, extras, and generic content with robust source path management, episode migration capabilities, and hierarchical data organization.

### Key Features

- **Hierarchical TV Structure**: Creates proper Show → Season → Episode → Version → Part hierarchy
- **Source Path Management**: Tracks and validates source paths to prevent conflicts
- **Temporary File System**: Uses temporary storage to handle complex scenarios where new episodes are added while existing episodes are edited
- **Duplicate Prevention**: Multiple layers prevent duplicate processing and database conflicts
- **Episode Migration**: Handles moving episodes between shows with 4-scenario logic
- **Crash Recovery**: Can resume scanning without duplicating work
- **Atomic Operations**: Processes entire source paths together for consistency

## Architecture

### Core Components

1. **Classifier** (`src-tauri/src/utils/classifier.rs`)
   - Media type detection and classification
   - Pattern matching for movies, TV episodes, and extras
   - Source folder determination

2. **Temporary File System** (`src-tauri/src/scanning/temp_files.rs`)
   - `TempFileManager`: Manages temporary storage during scanning
   - `SourcePathTracker`: Validates source path conflicts
   - `TempVideoItem`/`TempExtraItem`: Temporary data structures

3. **Video Scanning** (`src-tauri/src/scanning/video_scanning.rs`)
   - Main scanning orchestration
   - File processing and database operations
   - Episode migration logic

4. **Database Models** (`src-tauri/src/db/models.rs`)
   - `VideoItem`: Core media entity with `source_path` attribute
   - `VideoVersion`: Media versions/editions
   - `VideoPart`: Individual file parts

## Algorithm Flow

### 1. Initialization Phase

```rust
// Initialize temporary file manager and cleanup any existing files
let mut temp_manager = TempFileManager::new(index.id)?;
temp_manager.cleanup_existing_files()?;

// Initialize source path tracker
let mut source_tracker = SourcePathTracker::new();
```

**Purpose**: Clean up any leftover temporary files from crashed scans and initialize tracking systems.

### 2. Folder Processing Phase

The algorithm processes folders using a **breadth-first approach for files, depth-first for directories**:

```rust
// Process files in folder order before going deeper
while let Some(current_dir) = dirs_to_process.pop() {
    // First pass: collect all entries and separate files from directories
    let mut file_entries = Vec::new();
    let mut subdirs = Vec::new();
    
    // Process all files in the current directory first
    for entry_path in file_entries {
        // Process video file
    }
    
    // Then add subdirectories to stack for later processing
    for subdir in subdirs {
        dirs_to_process.push(subdir);
    }
}
```

**Purpose**: Ensures files are processed before descending deeper, enabling proper source path validation.

### 3. File Processing Phase

For each video file, the algorithm follows this sequence:

#### 3.1 Existing File Detection

```rust
// Check if video_part exists with same size + fast_hash
let existing_parts = video_repo.get_video_parts_by_size_and_hash(file_size, &fast_hash).await?;

if let Some(existing_part) = existing_parts.first() {
    // Handle existing file
}
```

**Purpose**: Detect if file already exists in database using fast hash and file size.

#### 3.2 Path Change Detection and Migration

If file exists but path has changed:

```rust
// Different path - check if this is a source path change that requires migration
let classified = classify_path(&file_path_str);

// Determine new source path based on classification
let new_source_path = match classified.media_type {
    MediaType::Movie => { /* movie folder detection logic */ }
    MediaType::TvEpisode => { /* TV source folder logic */ }
    _ => None,
};

// Check if source path has changed and handle migration if needed
if let Some(new_source_path) = new_source_path {
    if video_item.source_path.as_ref() != Some(&new_source_path) {
        handle_episode_migration(video_repo, existing_part.id, old_source_path, &new_source_path).await?;
    }
}
```

**Purpose**: Handle episode migration when files are moved between shows or folder structures change.

#### 3.3 New File Classification

For new files, the algorithm sets them aside for later processing if they are organized inside of a folder specific to that media (classifier designates a source_path in this case)

```rust
let classified = classify_path(&file_path_str);

match classified.media_type {
    MediaType::Extra => {
        // Add to extras temporary file
        temp_manager.add_extra(temp_extra)?;
    }
    MediaType::Movie => {
        if source_path.is_none() && source_tracker.get_source_paths().is_empty() {
            // Add movie immediately
            add_movie_immediately(...).await?;
        } else {
            // Add to temporary files for later processing
            temp_manager.add_new_content(temp_item)?;
        }
    }
    MediaType::TvEpisode | MediaType::Generic => {
        // Add to temporary files for later processing
        temp_manager.add_new_content(temp_item)?;
    }
}
```

**Purpose**: Classify new files and route them to appropriate processing paths.

### 4. Temporary File Processing Phase

The algorithm uses a sophisticated temporary file system to handle complex scenarios where new episodes are added while existing episodes are being edited. This ensures data consistency and prevents conflicts.

#### 4.1 Source Path-Based Processing

Files with source paths are stored in temporary files and processed when the entire source path is completed, ensuring updates happen before new content is added:

```rust
// During scanning - new files with source paths go to temporary storage
if let Some(source_path) = &source_path {
    source_tracker.track_source_path(source_path, &file_path_str)?;
    temp_manager.add_new_content(temp_item)?;
}

// Processing of files we already have a record of occur here

// When source path processing is complete, we add the new files to our records
if source_tracker.remove_source_path(path_to_remove) {
    println!("📝 Processing temporary files for completed source path: {}", path_to_remove);
    process_temp_files(temp_manager, video_repo, index_id).await?;
}
```

**Purpose**: 
- **Consistency**: All episodes from a show are processed together
- **Conflict Prevention**: Avoids conflicts when new episodes are added during existing episode edits

#### 4.2 Temporary File Processing Logic

When a source path is completed, the algorithm processes all temporary files:

```rust
async fn process_temp_files(
    temp_manager: &mut TempFileManager,
    video_repo: &VideoRepo,
    index_id: i64,
) -> Result<(), anyhow::Error> {
    // Process new content first
    let new_content = temp_manager.load_new_content()?;
    for item in new_content {
        process_temp_video_item(item, video_repo, index_id).await?;
    }
    
    // Process extras after new content
    let extras = temp_manager.load_extras()?;
    for item in extras {
        process_temp_extra_item(item, video_repo, index_id).await?;
    }
    
    // Clear temporary items after processing to prevent duplicates
    temp_manager.clear_items();
}
```

**Key Features**:
- **Duplicate Prevention**: Same files won't be processed multiple times
- **Memory Management**: Temporary items are cleared after processing
- **Batch Processing**: All content from a source path is processed together

#### 4.3 Final Cleanup Phase

After all source paths are processed, any remaining temporary files (content without source paths) are processed:

```rust
// Process any remaining temporary files (for content without source paths)
process_temp_files(&mut temp_manager, &video_repo, index.id).await?;
```

**Purpose**: Honestly unnecessary but leaving it in anyways out of paranoia

## Media Classification Logic

### Classification Order

The classifier follows a specific order of detection:

1. **Extras Detection** (highest priority as to not confuse it with movie editions)
2. **Numbered TV Episodes**
3. **Air Date Based TV Shows**
4. **Movies**
5. **Generic Content** (fallback)

### 1. Extras Detection

**Folder Names** (exact match, case insensitive):
- "behind the scenes", "deleted scenes", "interviews", "scenes"
- "samples", "shorts", "featurettes", "clips", "others", "extras", "trailers"

**Filename Suffixes** (exact match within string):
- "-behindthescenes", "-deleted", "-featurette", "-interview"
- "-scene", "-short", "-trailer", "-other"

### 2. Numbered TV Episodes

**SxEy Format**: `S(\d{1,3})E(\d{1,4})(?:-E?(\d{1,4}))?`
- Example: `S01E01`, `S02E15`, `S01E01-03`

**Season Folder + Ey/Epy**:
- Season folder pattern: `^season\s+(\d+)$`
- Episode patterns: `E(\d{1,4})` or `Ep(\d{1,4})`
- Specials folders: "special" or "specials" → season 0

### 3. Air Date Based TV Shows

**Date Patterns**:
- ISO format: `(\d{4})[-.](\d{1,2})[-.](\d{1,2})`
- DMY format: `(\d{1,2})[-.](\d{1,2})[-.](\d{4})`

### 4. Movies

**Year Patterns**:
- Parentheses: `(.+?)\s*\((\d{4})\)`
- Dots: `(.+?)\.(\d{4})`

**Version and Part Parsing**:
- Version: `{edition-version_name}`, ` - version_name`, ` - [version_name]`
- Part: ` - {part_type}#` where part_type ∈ {cd, dvd, part, pt, disc, disk}

### 5. Source Folder Determination

**For TV Episodes**:
- If in season folder → parent of season folder
- Otherwise → current folder

**For Movies**:
- Movie folder detection: check if movie title + year matches folder name
- If in matching folder → parent folder as source_path
- Otherwise → no source_path

## Episode Migration Logic

The algorithm handles 4 scenarios when episodes are moved between shows:

### Scenario 1: Old path doesn't exist, New path doesn't have video_item
```rust
(false, false) => {
    // Migrate old_path record to new path
    video_repo.update_video_item_source_path(video_item.id, Some(new_source_path.to_string())).await?;
}
```

### Scenario 2: Old path doesn't exist, New path already has a video_item
```rust
(false, true) => {
    // Move video_part and possibly video_version to new video_item
    let new_item = &new_path_items[0];
    move_video_part_to_item(video_repo, video_part_id, video_version.id, video_item.id, new_item.id).await?;
}
```

### Scenario 3: Old path still exists, New path doesn't have video_item
```rust
(true, false) => {
    // Create new video_item and move video_part and possible video_version to new video_item
    let new_item_id = video_repo.add_video_item(...).await?;
    move_video_part_to_item(video_repo, video_part_id, video_version.id, video_item.id, new_item_id).await?;
}
```

### Scenario 4: Old path still exists, New path already has a video_item
```rust
(true, true) => {
    // Move video_part and possibly video_version to new video_item
    let new_item = &new_path_items[0];
    move_video_part_to_item(video_repo, video_part_id, video_version.id, video_item.id, new_item.id).await?;
}
```

### Version Management

The `move_video_part_to_item` function handles version management intelligently:

- **Single Part Version**: Moves entire version to new item
- **Multi-Part Version**: Creates new version for just this part
- **Cleanup**: Deletes old video_item if it becomes empty

## Source Path Management

### Source Path Tracking

The `SourcePathTracker` validates source paths to prevent conflicts:

**Validation Rules:**
1. If no source path is set → set the incoming source path
2. If source path is already set and matches incoming → OK
3. If source path is already set and differs from incoming → ERROR

```rust
pub struct SourcePathTracker {
    source_path: Option<String>,
}

impl SourcePathTracker {
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
                        existing_path, source_path
                    ))
                }
            }
        }
    }
    
    /// Remove a source path from tracking (when finished processing a folder)
    /// Returns true if a source path was actually removed, false if none was set
    pub fn remove_source_path(&mut self, source_path: &str) -> bool {
        if self.source_path.as_ref().map_or(false, |path| path == source_path) {
            self.source_path = None;
            true
        } else {
            false
        }
    }
}
```

**Folder Completion**: After processing each folder, the source path is removed from tracking to prevent false conflicts in subsequent folders.

**Temporary File Processing**: When a source path is successfully removed from tracking, all temporary files for that source path are immediately processed and added to the database. This ensures that:
- Existing episodes are updated if the source path changed
- New content creates new video items with confidence that everything is new
- Extras are processed and added to the database
- Temporary files are cleaned up after processing

### TV Episode Hierarchy Processing

The algorithm creates a proper hierarchical structure for TV episodes: **Show → Season → Episode → Version → Part**

#### Hierarchy Creation Process

When processing TV episodes from temporary files, the algorithm follows this 5-step process:

```rust
async fn process_temp_tv_episode(
    item: &TempVideoItem,
    tv: TvEpisodeInfo,
    video_repo: &VideoRepo,
    index_id: i64,
) -> Result<(), anyhow::Error> {
    // Step 1: Find or create the show (video_item with source_path)
    let existing_shows = video_repo.get_video_items_by_source_path(index_id, &tv.source_path).await?;
    let show_id = if let Some(existing_show) = existing_shows.first() {
        existing_show.id
    } else {
        video_repo.add_video_item(index_id, "show", tv.show_name, None, Some(tv.source_path), metadata).await?
    };
    
    // Step 2: Find or create the season (video_item child of show)
    let existing_seasons = video_repo.get_video_items_by_parent_and_number(show_id, tv.season).await?;
    let season_id = if let Some(existing_season) = existing_seasons.first() {
        existing_season.id
    } else {
        video_repo.add_video_item_with_number(index_id, "season", season_title, Some(show_id), None, Some(tv.season), metadata).await?
    };
    
    // Step 3: Find or create the episode (video_item child of season)
    let existing_episodes = video_repo.get_video_items_by_parent_and_number(season_id, tv.episode).await?;
    let episode_id = if let Some(existing_episode) = existing_episodes.first() {
        existing_episode.id
    } else {
        video_repo.add_video_item_with_number(index_id, "episode", episode_title, Some(season_id), None, Some(tv.episode), metadata).await?
    };
    
    // Step 4: Create video version linked to the episode
    let version_id = video_repo.add_video_version_with_params(episode_id, tv.version, ...).await?;
    
    // Step 5: Create video part linked to the version
    video_repo.add_video_part_with_params(version_id, item.file_path, ...).await?;
}
```

#### Key Features

- **Number-Based Lookups**: Uses season/episode numbers instead of titles for reliable matching
- **Proper Hierarchy**: Creates Show → Season → Episode → Version → Part structure
- **Reuse Existing**: Finds existing seasons/episodes to avoid duplicates
- **Consistent Naming**: Season 0 = "Specials", others = "Season X"
- **Episode Titles**: Uses parsed episode title or falls back to "Episode X"

#### Database Methods

The algorithm uses specialized database methods for hierarchy management:

- `get_video_items_by_source_path()`: Find shows by source path
- `get_video_items_by_parent_and_number()`: Find seasons/episodes by parent and number
- `add_video_item_with_number()`: Create items with number field populated

### Movie Folder Detection

```rust
fn is_movie_in_matching_folder(movie_title: &str, movie_year: Option<i32>, folder_name: &str) -> bool {
    let normalize = |s: &str| s.replace(' ', "").replace('.', "").to_lowercase();
    
    let normalized_title = normalize(movie_title);
    let normalized_folder = normalize(folder_name);
    
    // Check if folder contains the movie title and year
    normalized_folder.contains(&normalized_title) && 
    movie_year.map_or(true, |year| normalized_folder.contains(&year.to_string()))
}
```

## Database Schema

### Video Items Table

```sql
CREATE TABLE video_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    index_id INTEGER NOT NULL,
    type TEXT NOT NULL,  -- Types: "movie", "show", "video", "extra"
    title TEXT NOT NULL,
    parent_id INTEGER,
    source_path TEXT,  -- Root folder for all content
    metadata TEXT NOT NULL,
    latest_added_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (index_id) REFERENCES indexes(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES video_items(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_video_items_source_path ON video_items(index_id, source_path);
```

**Video Item Types:**
- `"movie"`: Movies and films
- `"show"`: TV shows and series
- `"season"`: TV show seasons
- `"episode"`: Individual TV episodes
- `"video"`: Generic video content
- `"extra"`: Behind-the-scenes, trailers, featurettes, etc.

## Error Handling

### Crash Recovery

The algorithm includes robust crash recovery:

1. **Temporary File Cleanup**: Removes leftover files from crashed scans
2. **Status Recovery**: Restarts indexes with "scanning" status
3. **Atomic Operations**: Database operations are designed to be safe

### Conflict Detection

- **Source Path Conflicts**: Detected during scanning
- **File Hash Conflicts**: Handled by updating existing records
- **Migration Conflicts**: Resolved using the 4-scenario logic

## Performance Considerations

### Temporary File System

- **Memory Storage**: Uses in-memory vectors instead of disk serialization
- **Batch Processing**: Processes all files after scanning complete
- **Efficient Cleanup**: Single cleanup operation at end

### Database Optimization

- **Indexes**: Added on `source_path` for efficient lookups
- **Batch Operations**: Groups database writes for better performance
- **Hash-based Detection**: Fast file identification using fast hash

## Testing

### Unit Tests

The classifier includes comprehensive unit tests:

```rust
#[test]
fn test_extra_folder_detection() {
    let result = classify_path("Movies/Avatar/Behind The Scenes/Making Of.mkv");
    assert_eq!(result.media_type, MediaType::Extra);
}

#[test]
fn test_tv_sxxeyy() {
    let result = classify_path("TV/Some Show/Season 1/Some.Show.S01E01.mkv");
    assert_eq!(result.media_type, MediaType::TvEpisode);
    let tv = result.tv_episode.unwrap();
    assert_eq!(tv.season, 1);
    assert_eq!(tv.episode, 1);
}
```

## Future Enhancements

### Planned Improvements

1. **Extras Association**: Link extras to their parent content
2. **Metadata Extraction**: Enhanced metadata from filenames
3. **External ID Integration**: Proper handling of IMDb, TMDB, TVDB IDs
4. **Conflict Resolution**: More sophisticated conflict resolution
5. **Performance Optimization**: Parallel processing capabilities

### Extension Points

- **Custom Classifiers**: Plugin system for custom media types
- **Metadata Providers**: Integration with external metadata services
- **Notification System**: Real-time updates for scanning progress

## Conclusion

The video scanning algorithm provides a robust, scalable solution for managing complex media libraries. It handles real-world scenarios including folder restructuring, episode migration, and crash recovery while maintaining data integrity and performance.

The modular design allows for easy extension and customization while the comprehensive error handling ensures reliable operation in production environments.
