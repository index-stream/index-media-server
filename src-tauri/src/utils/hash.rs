use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use xxhash_rust::xxh3::xxh3_128;

/// Calculate fast hash for a file using xxHash128
/// For files < 40MB: hash the entire file
/// For files >= 40MB: hash 5 evenly spaced segments (first 4MB, 3 evenly spaced 4MB segments, last 4MB)
pub async fn calculate_fast_hash(file_path: &Path) -> Result<String, anyhow::Error> {
    let mut file = File::open(file_path)?;
    let file_size = file.metadata()?.len();
    
    // If file is under 40MB, hash the entire file
    if file_size < 40 * 1024 * 1024 {
        return hash_entire_file(&mut file).await;
    }
    
    // For files >= 40MB, hash 5 segments
    hash_file_segments(&mut file, file_size).await
}

/// Hash the entire file
async fn hash_entire_file(file: &mut File) -> Result<String, anyhow::Error> {
    file.seek(SeekFrom::Start(0))?;
    
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let hash = xxh3_128(&buffer);
    Ok(format!("{:032x}", hash))
}

/// Hash 5 segments of a large file (evenly spaced segments)
async fn hash_file_segments(file: &mut File, file_size: u64) -> Result<String, anyhow::Error> {
    const SEGMENT_SIZE: u64 = 4 * 1024 * 1024; // 4MB
    let mut combined_data = Vec::new();
    
    // Calculate evenly spaced segment positions
    // For 5 segments, we need 4 gaps between them
    // Total space available for gaps = file_size - SEGMENT_SIZE (last segment)
    // Each gap = (file_size - SEGMENT_SIZE) / 4
    let gap_size = (file_size - SEGMENT_SIZE) / 4;
    
    // Segment 1: First 4MB (at position 0)
    file.seek(SeekFrom::Start(0))?;
    let mut buffer = vec![0u8; SEGMENT_SIZE as usize];
    file.read_exact(&mut buffer)?;
    combined_data.extend_from_slice(&buffer);
    
    // Segments 2-4: Three evenly spaced segments
    for i in 1..4 {
        let start_pos = i * gap_size;
        file.seek(SeekFrom::Start(start_pos))?;
        
        let mut buffer = vec![0u8; SEGMENT_SIZE as usize];
        file.read_exact(&mut buffer)?;
        combined_data.extend_from_slice(&buffer);
    }
    
    // Segment 5: Last 4MB
    file.seek(SeekFrom::Start(file_size - SEGMENT_SIZE))?;
    let mut buffer = vec![0u8; SEGMENT_SIZE as usize];
    file.read_exact(&mut buffer)?;
    combined_data.extend_from_slice(&buffer);
    
    let hash = xxh3_128(&combined_data);
    Ok(format!("{:032x}", hash))
}
