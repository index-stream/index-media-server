use crate::models::config::{IncomingMediaIndex, IndexUpdateRequest};
use crate::api::responses::IndexResponse;
use crate::db::repos::IndexesRepo;
use crate::api::state::AppState;
use crate::config::{config_path, icons_dir};
use crate::utils::image::detect_image_extension;
use base64::{Engine as _, engine::general_purpose};
use tokio::fs;
use warp::reject::custom;

// Custom error types for index operations
#[derive(Debug)]
pub struct IndexError;

impl warp::reject::Reject for IndexError {}

// Handler for getting all indexes
pub async fn handle_get_indexes(
    app_state: AppState,
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Get all indexes from database
    let indexes_repo = IndexesRepo::new(app_state.db_pool.clone());
    
    let indexes: Vec<IndexResponse> = indexes_repo.get_all_indexes().await
        .map_err(|e| {
            eprintln!("Failed to fetch indexes: {}", e);
            custom(IndexError)
        })?
        .into_iter()
        .map(IndexResponse::from)
        .collect();
    
    let response = serde_json::json!({
        "indexes": indexes
    });
    
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        warp::http::StatusCode::OK,
    ))
}

// Handler for creating a new local index
pub async fn handle_create_local_index(
    app_state: AppState,
    index_request: IncomingMediaIndex,
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Validate index name is not empty
    if index_request.name.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Index name is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if index_request.r#type.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Type is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| custom(IndexError))?;
    
    let icons_dir = icons_dir(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get icons directory: {}", e);
            custom(IndexError)
        })?;
    
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            custom(IndexError)
        })?;
    
    // Create directories if they don't exist
    fs::create_dir_all(&icons_dir).await
        .map_err(|e| {
            eprintln!("Failed to create icons directory: {}", e);
            custom(IndexError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            custom(IndexError)
        })?;
    
    let _config: crate::models::config::Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            custom(IndexError)
        })?;
    
    // Create index in database first to get the auto-increment ID
    let indexes_repo = IndexesRepo::new(app_state.db_pool.clone());
    
    // Prepare metadata for the index
    let metadata = serde_json::json!({
        "folders": index_request.folders,
    });
    
    let index_id = indexes_repo.add_index(
        index_request.name.trim().to_string(),
        index_request.r#type.trim().to_string(),
        Some(index_request.icon.clone()),
        metadata,
    ).await
        .map_err(|e| {
            eprintln!("Failed to create index: {}", e);
            custom(IndexError)
        })?;
    
    // Handle custom icon files if present - now using the database ID
    if let Some(custom_icon_data) = index_request.custom_icon_file {
        // Decode base64 data
        let icon_data = general_purpose::STANDARD.decode(custom_icon_data)
            .map_err(|e| {
                eprintln!("Failed to decode custom icon: {}", e);
                custom(IndexError)
            })?;
        
        // Detect image format and get appropriate extension
        let extension = detect_image_extension(&icon_data)
            .map_err(|e| {
                eprintln!("Failed to detect image format: {}", e);
                custom(IndexError)
            })?;
        
        // Save with correct extension using the database index ID
        let icon_path = icons_dir.join(format!("index_{}.{}", index_id, extension));
        fs::write(&icon_path, icon_data).await
            .map_err(|e| {
                eprintln!("Failed to save custom icon: {}", e);
                custom(IndexError)
            })?;
        
        println!("Saved custom icon for index '{}' with ID '{}' as {} to: {:?}", 
                 index_request.name, index_id, extension, icon_path);
    }
    
    // Get the created index to return in response
    let created_index = indexes_repo.get_index_by_id(index_id).await
        .map_err(|e| {
            eprintln!("Failed to fetch created index: {}", e);
            custom(IndexError)
        })?
        .ok_or_else(|| {
            eprintln!("Created index not found");
            custom(IndexError)
        })?;
    
    println!("Local index '{}' created successfully with ID: {}", created_index.name, created_index.id);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Local index created successfully",
            "index": IndexResponse::from(created_index)
        })),
        warp::http::StatusCode::CREATED,
    ))
}

// Handler for updating an index
pub async fn handle_update_index(
    app_state: AppState,
    index_id: String,
    index_request: IndexUpdateRequest,
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Validate index name is not empty
    if index_request.name.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Index name is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if index_request.r#type.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Type is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| custom(IndexError))?;
    
    let icons_dir = icons_dir(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get icons directory: {}", e);
            custom(IndexError)
        })?;
    
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            custom(IndexError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            custom(IndexError)
        })?;
    
    let _config: crate::models::config::Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            custom(IndexError)
        })?;
    
    // Update index in database
    let indexes_repo = IndexesRepo::new(app_state.db_pool.clone());
    
    // Parse index_id as i64
    let index_id = index_id.parse::<i64>()
        .map_err(|_| {
            eprintln!("Invalid index ID format");
            custom(IndexError)
        })?;
    
    // Check if index exists
    let existing_index = indexes_repo.get_index_by_id(index_id).await
        .map_err(|e| {
            eprintln!("Failed to fetch index: {}", e);
            custom(IndexError)
        })?
        .ok_or_else(|| {
            eprintln!("Index not found");
            custom(IndexError)
        })?;
    
    // Handle custom icon files if present - using database ID
    if let Some(custom_icon_data) = index_request.custom_icon_file {
        // Decode base64 data
        let icon_data = general_purpose::STANDARD.decode(custom_icon_data)
            .map_err(|e| {
                eprintln!("Failed to decode custom icon: {}", e);
                custom(IndexError)
            })?;
        
        // Detect image format and get appropriate extension
        let extension = detect_image_extension(&icon_data)
            .map_err(|e| {
                eprintln!("Failed to detect image format: {}", e);
                custom(IndexError)
            })?;
        
        // Save with correct extension using the database index ID
        let icon_path = icons_dir.join(format!("index_{}.{}", index_id, extension));
        fs::write(&icon_path, icon_data).await
            .map_err(|e| {
                eprintln!("Failed to save custom icon: {}", e);
                custom(IndexError)
            })?;
        
        println!("Updated custom icon for index '{}' with ID '{}' as {} to: {:?}", 
                 existing_index.name, index_id, extension, icon_path);
    }
    
    // Prepare updated metadata
    let metadata = serde_json::json!({
        "folders": index_request.folders,
    });
    
    // Update the index
    indexes_repo.update_index(
        index_id,
        index_request.name.trim().to_string(),
        Some(index_request.icon.trim().to_string()),
        metadata,
    ).await
        .map_err(|e| {
            eprintln!("Failed to update index: {}", e);
            custom(IndexError)
        })?;
    
    // Get the updated index to return in response
    let updated_index = indexes_repo.get_index_by_id(index_id).await
        .map_err(|e| {
            eprintln!("Failed to fetch updated index: {}", e);
            custom(IndexError)
        })?
        .ok_or_else(|| {
            eprintln!("Updated index not found");
            custom(IndexError)
        })?;
    
    println!("Index '{}' updated successfully", updated_index.name);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Index updated successfully",
            "index": IndexResponse::from(updated_index)
        })),
        warp::http::StatusCode::OK,
    ))
}

// Handler for deleting an index
pub async fn handle_delete_index(
    app_state: AppState,
    index_id: String,
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Get the app handle from app state
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| custom(IndexError))?.clone();
    drop(app_handle_guard); // Release the lock
    
    let config_path = config_path(&app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            custom(IndexError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            custom(IndexError)
        })?;
    
    let _config: crate::models::config::Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            custom(IndexError)
        })?;
    
    // Delete index from database
    let indexes_repo = IndexesRepo::new(app_state.db_pool.clone());
    
    // Parse index_id as i64
    let index_id = index_id.parse::<i64>()
        .map_err(|_| {
            eprintln!("Invalid index ID format");
            custom(IndexError)
        })?;
    
    // Check if index exists before deleting
    let existing_index = indexes_repo.get_index_by_id(index_id).await
        .map_err(|e| {
            eprintln!("Failed to fetch index: {}", e);
            custom(IndexError)
        })?
        .ok_or_else(|| {
            eprintln!("Index not found");
            custom(IndexError)
        })?;
    
    // Try to remove associated icon file if it exists
    let icons_dir = icons_dir(&app_handle)
        .map_err(|e| {
            eprintln!("Failed to get icons directory: {}", e);
            custom(IndexError)
        })?;
    let icon_extensions = ["png", "jpg", "jpeg", "gif", "bmp", "webp"];
    for ext in &icon_extensions {
        let icon_path = icons_dir.join(format!("index_{}.{}", index_id, ext));
        if icon_path.exists() {
            if let Err(e) = fs::remove_file(&icon_path).await {
                eprintln!("Warning: Failed to remove icon file {:?}: {}", icon_path, e);
            } else {
                println!("Removed icon file: {:?}", icon_path);
            }
        }
    }
    
    // Delete the index
    indexes_repo.delete_index(index_id).await
        .map_err(|e| {
            eprintln!("Failed to delete index: {}", e);
            custom(IndexError)
        })?;
    
    println!("Index '{}' deleted successfully", existing_index.name);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Index deleted successfully"
        })),
        warp::http::StatusCode::OK,
    ))
}
