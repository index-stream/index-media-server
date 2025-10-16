use crate::api::state::AppState;
use crate::db::repos::IndexesRepo;
use crate::scanning::video_scanning::scan_video_index;
use std::time::Duration;
use tokio::time::sleep;

/// Background scanning process that continuously scans indexes
pub async fn start_scanning_process(app_state: AppState) {
    println!("🔍 Starting background scanning process...");
    
    loop {
        let scanned = match process_scanning_cycle(&app_state).await {
            Ok(scanned) => scanned,
            Err(e) => {
                eprintln!("Error in scanning cycle: {}", e);
                sleep(Duration::from_secs(30)).await;
                true
            }
        };
        
        if !scanned {
            // No indexes were scanned, wait 30 seconds before checking again
            println!("⏳ No indexes to scan, waiting 30 seconds...");
            sleep(Duration::from_secs(30)).await;
        }
        // If we did scan something, immediately start the next cycle
    }
}

/// Process one scanning cycle: check for scanning/queued indexes and scan them
async fn process_scanning_cycle(app_state: &AppState) -> Result<bool, anyhow::Error> {
    let indexes_repo = IndexesRepo::new(app_state.db_pool.clone());
    
    // First, check if there are any indexes with status "scanning" (recovery from crash)
    let scanning_indexes = indexes_repo.get_indexes_by_scan_status("scanning").await?;
    
    if !scanning_indexes.is_empty() {
        println!("🔄 Found {} index(es) with 'scanning' status - recovering from previous session", scanning_indexes.len());
        
        for index in scanning_indexes {
            println!("🔄 Restarting scan for index '{}' (ID: {})", index.name, index.id);
            match index.r#type.as_str() {
                "videos" => {
                    scan_video_index(&indexes_repo, &index, app_state).await?;
                }
                _ => {
                    println!("⚠️  Index type '{}' not supported yet for index '{}' (ID: {})", index.r#type, index.name, index.id);
                    indexes_repo.update_scan_status(index.id, "failed".to_string()).await?;
                }
            }
        }
        return Ok(true);
    }
    
    // No scanning indexes found, check for queued indexes
    let queued_indexes = indexes_repo.get_indexes_by_scan_status("queued").await?;
    
    if queued_indexes.is_empty() {
        println!("📭 No queued indexes found");
        return Ok(false);
    }
    
    // Find the queued index with the oldest last_scanned_at
    let oldest_index = queued_indexes
        .into_iter()
        .min_by_key(|index| index.last_scanned_at)
        .expect("At least one queued index should exist");
    
    println!("📋 Found queued index '{}' (ID: {}) with oldest last_scanned_at: {}", 
             oldest_index.name, oldest_index.id, oldest_index.last_scanned_at);
    
    // Set status to scanning
    indexes_repo.update_scan_status(oldest_index.id, "scanning".to_string()).await?;
    
    // Scan the index
    match oldest_index.r#type.as_str() {
        "videos" => {
            if let Err(e) = scan_video_index(&indexes_repo, &oldest_index, app_state).await {
                eprintln!("❌ Failed to scan index '{}' (ID: {}): {}", oldest_index.name, oldest_index.id, e);
                // Reset status back to failed so it can be retried later
                if let Err(reset_err) = indexes_repo.update_scan_status(oldest_index.id, "failed".to_string()).await {
                    eprintln!("❌ Failed to reset scan status for index '{}' (ID: {}): {}", oldest_index.name, oldest_index.id, reset_err);
                }
                return Ok(false); // Return false so we wait before trying again
            }
        }
        _ => {
            println!("⚠️  Index type '{}' not supported yet for index '{}' (ID: {})", oldest_index.r#type, oldest_index.name, oldest_index.id);
            if let Err(e) = indexes_repo.update_scan_status(oldest_index.id, "failed".to_string()).await {
                eprintln!("❌ Failed to set scan status to 'failed' for index '{}' (ID: {}): {}", oldest_index.name, oldest_index.id, e);
            }
            return Ok(false); // Return false so we wait before trying again
        }
    }
    
    Ok(true)
}
