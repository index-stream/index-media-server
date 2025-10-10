use crate::models::config::{Configuration, IncomingConfiguration, MediaIndex, ServerPasswordUpdate, ServerNameUpdate, ConfigurationResponse, IncomingProfile, IncomingMediaIndex, IndexUpdateRequest};
use crate::utils::image::detect_image_extension;
use crate::api::state::AppState;
use crate::config::{config_path, icons_dir};
use base64::{Engine as _, engine::general_purpose};
use tokio::fs;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use std::path::PathBuf;

// Helper function to hash a password using Argon2id
fn hash_password(password: &str) -> Result<String, String> {
    if password.is_empty() {
        return Ok(String::new());
    }
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    
    Ok(password_hash.to_string())
}

// Handler for getting configuration
pub async fn handle_get_configuration(app_state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigGetError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    // Try to read the configuration file
    match fs::read_to_string(&config_path).await {
        Ok(config_json) => {
            // Parse the JSON to validate it's valid
            match serde_json::from_str::<Configuration>(&config_json) {
                Ok(config) => {
                    let config_response = ConfigurationResponse::from(config);
                    Ok(warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({
                            "config": config_response
                        })),
                        warp::http::StatusCode::OK,
                    ))
                },
                Err(e) => {
                    eprintln!("Failed to parse configuration JSON: {}", e);
                    Ok(warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({
                            "error": "Invalid configuration format"
                        })),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Configuration file doesn't exist, return 404
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Configuration not found"
                })),
                warp::http::StatusCode::NOT_FOUND,
            ))
        }
        Err(e) => {
            eprintln!("Failed to read configuration file: {}", e);
            Ok(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "error": "Failed to read configuration"
                })),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

// Handler for saving configuration
pub async fn handle_save_configuration(
    app_state: AppState,
    incoming_config: IncomingConfiguration,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?;
    
    // Get the config file path and icons directory using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    let icons_dir = icons_dir(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get icons directory: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Process custom icons and convert to final configuration
    let mut final_indexes = Vec::new();
    
    for incoming_index in incoming_config.indexes {
        let index_id = incoming_index.id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let final_index = MediaIndex {
            id: index_id.clone(),
            name: incoming_index.name,
            media_type: incoming_index.media_type,
            icon: incoming_index.icon,
            folders: incoming_index.folders,
            r#type: incoming_index.r#type,
        };
        
        // Handle custom icon files if present
        if let Some(custom_icon_data) = incoming_index.custom_icon_file {
            // Decode base64 data
            let icon_data = general_purpose::STANDARD.decode(custom_icon_data)
                .map_err(|e| {
                    eprintln!("Failed to decode custom icon: {}", e);
                    warp::reject::custom(ConfigSaveError)
                })?;
            
            // Detect image format and get appropriate extension
            let extension = detect_image_extension(&icon_data)
                .map_err(|e| {
                    eprintln!("Failed to detect image format: {}", e);
                    warp::reject::custom(ConfigSaveError)
                })?;
            
            // Save with correct extension using the index id
            let icon_path = icons_dir.join(format!("{}.{}", index_id, extension));
            fs::write(&icon_path, icon_data).await
                .map_err(|e| {
                    eprintln!("Failed to save custom icon: {}", e);
                    warp::reject::custom(ConfigSaveError)
                })?;
            
            println!("Saved custom icon for index '{}' with ID '{}' as {} to: {:?}", 
                     final_index.name, index_id, extension, icon_path);
        }
        
        final_indexes.push(final_index);
    }
    
    // Create final configuration
    let final_config = Configuration {
        id: Uuid::new_v4().to_string(),
        name: incoming_config.name,
        profiles: incoming_config.profiles.into_iter().map(|p| crate::models::config::Profile {
            id: p.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: p.name,
            color: p.color,
        }).collect(),
        password: hash_password(&incoming_config.password)
            .map_err(|e| {
                eprintln!("Failed to hash password: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?,
        indexes: final_indexes,
    };
    
    // Save the configuration as JSON
    let config_json = serde_json::to_string_pretty(&final_config)
        .map_err(|e| {
            eprintln!("Failed to serialize configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    fs::write(&config_path, config_json).await
        .map_err(|e| {
            eprintln!("Failed to save configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    println!("Configuration saved successfully to: {:?}", config_path);
    
    // Convert to response format (excluding password)
    let config_response = ConfigurationResponse::from(final_config);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Configuration saved successfully",
            "config": config_response
        })),
        warp::http::StatusCode::OK,
    ))
}

// Handler for updating server password
pub async fn handle_update_server_password(
    app_state: AppState,
    password_update: ServerPasswordUpdate,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate password is not empty
    if password_update.password.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Password is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    let mut config: Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Update password
    config.password = hash_password(&password_update.password)
        .map_err(|e| {
            eprintln!("Failed to hash password: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Save updated configuration
    let updated_config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| {
            eprintln!("Failed to serialize configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    fs::write(&config_path, updated_config_json).await
        .map_err(|e| {
            eprintln!("Failed to save configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    println!("Server password updated successfully");
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Server password updated successfully"
        })),
        warp::http::StatusCode::OK,
    ))
}

// Handler for updating server name
pub async fn handle_update_server_name(
    app_state: AppState,
    name_update: ServerNameUpdate,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate name is not empty
    if name_update.name.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Server name is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    let mut config: Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Update name
    config.name = name_update.name.trim().to_string();
    
    // Save updated configuration
    let updated_config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| {
            eprintln!("Failed to serialize configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    fs::write(&config_path, updated_config_json).await
        .map_err(|e| {
            eprintln!("Failed to save configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    println!("Server name updated successfully to: {}", config.name);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Server name updated successfully"
        })),
        warp::http::StatusCode::OK,
    ))
}

// Custom error types
#[derive(Debug)]
pub struct ConfigGetError;

#[derive(Debug)]
pub struct ConfigNotFoundError;

#[derive(Debug)]
pub struct ConfigSaveError;

// Handler for creating a new profile
pub async fn handle_create_profile(
    app_state: AppState,
    profile_request: IncomingProfile,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate input
    if profile_request.name.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Profile name is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if profile_request.color.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Profile color is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    let mut config: Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Create new profile with generated ID
    let new_profile = crate::models::config::Profile {
        id: Uuid::new_v4().to_string(),
        name: profile_request.name.trim().to_string(),
        color: profile_request.color.trim().to_string(),
    };
    
    // Add profile to configuration
    config.profiles.push(new_profile.clone());
    
    // Save updated configuration
    let updated_config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| {
            eprintln!("Failed to serialize configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    fs::write(&config_path, updated_config_json).await
        .map_err(|e| {
            eprintln!("Failed to save configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    println!("Profile '{}' created successfully with ID: {}", new_profile.name, new_profile.id);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Profile created successfully",
            "profile": new_profile
        })),
        warp::http::StatusCode::CREATED,
    ))
}

// Handler for updating an existing profile
pub async fn handle_update_profile(
    app_state: AppState,
    profile_id: String,
    profile_request: IncomingProfile,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate input
    if profile_request.name.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Profile name is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if profile_request.color.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Profile color is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    let mut config: Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Find and update the profile
    if let Some(profile_index) = config.profiles.iter().position(|p| p.id == profile_id) {
        let profile = &mut config.profiles[profile_index];
        profile.name = profile_request.name.trim().to_string();
        profile.color = profile_request.color.trim().to_string();
        
        let updated_profile_name = profile.name.clone();
        let updated_profile = profile.clone();
        
        // Drop the mutable reference before serializing
        let _ = profile;
        
        // Save updated configuration
        let updated_config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| {
                eprintln!("Failed to serialize configuration: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        fs::write(&config_path, updated_config_json).await
            .map_err(|e| {
                eprintln!("Failed to save configuration: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        println!("Profile '{}' updated successfully", updated_profile_name);
        
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": true,
                "message": "Profile updated successfully",
                "profile": updated_profile
            })),
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Profile not found"
            })),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

// Handler for deleting a profile
pub async fn handle_delete_profile(
    app_state: AppState,
    profile_id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    let mut config: Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Find and remove the profile
    if let Some(index) = config.profiles.iter().position(|p| p.id == profile_id) {
        let removed_profile = config.profiles.remove(index);
        
        // Save updated configuration
        let updated_config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| {
                eprintln!("Failed to serialize configuration: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        fs::write(&config_path, updated_config_json).await
            .map_err(|e| {
                eprintln!("Failed to save configuration: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        println!("Profile '{}' deleted successfully", removed_profile.name);
        
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": true,
                "message": "Profile deleted successfully"
            })),
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Profile not found"
            })),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

// Handler for creating a new local index
pub async fn handle_create_local_index(
    app_state: AppState,
    index_request: IncomingMediaIndex,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate input
    if index_request.name.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Index name is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    if index_request.media_type.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Media type is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get the app handle from app state
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?.clone();
    drop(app_handle_guard); // Release the lock
    
    let icons_dir = crate::config::icons_dir(&app_handle)
        .map_err(|e| {
            eprintln!("Failed to get icons directory: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    let config_path = crate::config::config_path(&app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Create directories if they don't exist
    fs::create_dir_all(&icons_dir).await
        .map_err(|e| {
            eprintln!("Failed to create icons directory: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    let mut config: Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Generate new index ID
    let index_id = Uuid::new_v4().to_string();
    
    // Handle custom icon files if present
    let final_icon = if let Some(custom_icon_data) = index_request.custom_icon_file {
        // Decode base64 data
        let icon_data = general_purpose::STANDARD.decode(custom_icon_data)
            .map_err(|e| {
                eprintln!("Failed to decode custom icon: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        // Detect image format and get appropriate extension
        let extension = detect_image_extension(&icon_data)
            .map_err(|e| {
                eprintln!("Failed to detect image format: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        // Save with correct extension using the index id
        let icon_path = icons_dir.join(format!("{}.{}", index_id, extension));
        fs::write(&icon_path, icon_data).await
            .map_err(|e| {
                eprintln!("Failed to save custom icon: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        println!("Saved custom icon for index '{}' with ID '{}' as {} to: {:?}", 
                 index_request.name, index_id, extension, icon_path);
        
        index_request.icon.clone()
    } else {
        index_request.icon.clone()
    };
    
    // Create new index
    let new_index = MediaIndex {
        id: index_id.clone(),
        name: index_request.name.trim().to_string(),
        media_type: index_request.media_type.trim().to_string(),
        icon: final_icon,
        folders: index_request.folders,
        r#type: index_request.r#type,
    };
    
    // Add index to configuration
    config.indexes.push(new_index.clone());
    
    // Save updated configuration
    let updated_config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| {
            eprintln!("Failed to serialize configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    fs::write(&config_path, updated_config_json).await
        .map_err(|e| {
            eprintln!("Failed to save configuration: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    println!("Local index '{}' created successfully with ID: {}", new_index.name, new_index.id);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Local index created successfully",
            "index": new_index
        })),
        warp::http::StatusCode::CREATED,
    ))
}

// Handler for updating an existing index
pub async fn handle_update_index(
    app_state: AppState,
    index_id: String,
    index_request: IndexUpdateRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Validate input
    if index_request.name.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Index name is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get the app handle from app state
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?.clone();
    drop(app_handle_guard); // Release the lock
    
    let icons_dir = crate::config::icons_dir(&app_handle)
        .map_err(|e| {
            eprintln!("Failed to get icons directory: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    let config_path = crate::config::config_path(&app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    let mut config: Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Find and update the index
    if let Some(index_index) = config.indexes.iter().position(|i| i.id == index_id) {
        let index = &mut config.indexes[index_index];
        
        // Update fields (excluding mediaType)
        index.name = index_request.name.trim().to_string();
        index.icon = index_request.icon.trim().to_string();
        index.folders = index_request.folders;
        
        // Handle custom icon files if present
        if let Some(custom_icon_data) = index_request.custom_icon_file {
            // Decode base64 data
            let icon_data = general_purpose::STANDARD.decode(custom_icon_data)
                .map_err(|e| {
                    eprintln!("Failed to decode custom icon: {}", e);
                    warp::reject::custom(ConfigSaveError)
                })?;
            
            // Detect image format and get appropriate extension
            let extension = detect_image_extension(&icon_data)
                .map_err(|e| {
                    eprintln!("Failed to detect image format: {}", e);
                    warp::reject::custom(ConfigSaveError)
                })?;
            
            // Save with correct extension using the index id
            let icon_path = icons_dir.join(format!("{}.{}", index_id, extension));
            fs::write(&icon_path, icon_data).await
                .map_err(|e| {
                    eprintln!("Failed to save custom icon: {}", e);
                    warp::reject::custom(ConfigSaveError)
                })?;
            
            println!("Updated custom icon for index '{}' with ID '{}' as {} to: {:?}", 
                     index.name, index_id, extension, icon_path);
        }
        
        let updated_index_name = index.name.clone();
        let updated_index = index.clone();
        
        // Drop the mutable reference before serializing
        let _ = index;
        
        // Save updated configuration
        let updated_config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| {
                eprintln!("Failed to serialize configuration: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        fs::write(&config_path, updated_config_json).await
            .map_err(|e| {
                eprintln!("Failed to save configuration: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        println!("Index '{}' updated successfully", updated_index_name);
        
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": true,
                "message": "Index updated successfully",
                "index": updated_index
            })),
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Index not found"
            })),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

// Handler for deleting an index
pub async fn handle_delete_index(
    app_state: AppState,
    index_id: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Get the app handle from app state
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?.clone();
    drop(app_handle_guard); // Release the lock
    
    let config_path = crate::config::config_path(&app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    let mut config: Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Find and remove the index
    if let Some(index_pos) = config.indexes.iter().position(|i| i.id == index_id) {
        let removed_index = config.indexes.remove(index_pos);
        
        // Try to remove associated icon file if it exists
        let icons_dir = crate::config::icons_dir(&app_handle)
            .map_err(|e| {
                eprintln!("Failed to get icons directory: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        let icon_extensions = ["png", "jpg", "jpeg", "gif", "bmp", "webp"];
        for ext in &icon_extensions {
            let icon_path = icons_dir.join(format!("{}.{}", index_id, ext));
            if icon_path.exists() {
                if let Err(e) = fs::remove_file(&icon_path).await {
                    eprintln!("Warning: Failed to remove icon file {:?}: {}", icon_path, e);
                } else {
                    println!("Removed icon file: {:?}", icon_path);
                }
            }
        }
        
        // Save updated configuration
        let updated_config_json = serde_json::to_string_pretty(&config)
            .map_err(|e| {
                eprintln!("Failed to serialize configuration: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        fs::write(&config_path, updated_config_json).await
            .map_err(|e| {
                eprintln!("Failed to save configuration: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
        
        println!("Index '{}' deleted successfully", removed_index.name);
        
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": true,
                "message": "Index deleted successfully"
            })),
            warp::http::StatusCode::OK,
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Index not found"
            })),
            warp::http::StatusCode::NOT_FOUND,
        ))
    }
}

// Handler for serving custom icons by index ID
pub async fn handle_get_index_icon(index_id: String) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // Validate index ID
    if index_id.trim().is_empty() {
        return Ok(Box::new(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Index ID is required"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        )));
    }

    // Use the global app handle to get the icons directory
    let app_handle = crate::api::controllers::icon::get_app_handle()
        .ok_or_else(|| warp::reject::custom(ConfigGetError))?;
    let icons_dir = crate::config::icons_dir(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get icons directory: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;

    // Try to find the icon file with common image extensions
    let extensions = ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico"];
    let mut icon_path: Option<PathBuf> = None;
    let mut found_extension: Option<&str> = None;

    for ext in &extensions {
        let test_path = icons_dir.join(format!("{}.{}", index_id, ext));
        if test_path.exists() {
            icon_path = Some(test_path);
            found_extension = Some(ext);
            break;
        }
    }

    match icon_path {
        Some(path) => {
            // Read the icon file
            let icon_data = fs::read(&path).await
                .map_err(|e| {
                    eprintln!("Failed to read icon file: {}", e);
                    warp::reject::custom(ConfigGetError)
                })?;

            // Determine content type based on extension
            let content_type = match found_extension.unwrap_or("png") {
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "webp" => "image/webp",
                "svg" => "image/svg+xml",
                "bmp" => "image/bmp",
                "ico" => "image/x-icon",
                _ => "image/png", // Default fallback
            };

            // Return the image data with appropriate content type
            let mut response = warp::reply::Response::new(icon_data.into());
            response.headers_mut().insert(
                "content-type",
                warp::http::HeaderValue::from_static(content_type),
            );
            response.headers_mut().insert(
                "cache-control",
                warp::http::HeaderValue::from_static("public, max-age=3600"),
            );
            Ok(Box::new(response))
        }
        None => {
            // Return 404 if no icon found
            Ok(Box::new(warp::reply::with_status(
                warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": "Icon not found"
                })),
                warp::http::StatusCode::NOT_FOUND,
            )))
        }
    }
}

impl warp::reject::Reject for ConfigGetError {}
impl warp::reject::Reject for ConfigNotFoundError {}
impl warp::reject::Reject for ConfigSaveError {}
