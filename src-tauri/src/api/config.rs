use crate::models::config::{Configuration, IncomingConfiguration, ServerPasswordUpdate, ServerNameUpdate, IncomingProfile, IncomingMediaIndex};
use crate::api::responses::{DatabaseConfigurationResponse, ProfileResponse, IndexResponse};
use crate::db::repos::{ProfilesRepo, IndexesRepo};
use crate::api::state::AppState;
use crate::config::config_path;
use crate::api::{profiles, indexes};
use tokio::fs;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{rand_core::OsRng, SaltString};

// Custom error types
#[derive(Debug)]
pub struct ConfigNotFoundError;

impl warp::reject::Reject for ConfigNotFoundError {}

#[derive(Debug)]
pub struct ConfigSaveError;

impl warp::reject::Reject for ConfigSaveError {}

#[derive(Debug)]
pub struct ConfigGetError;

impl warp::reject::Reject for ConfigGetError {}

// Handler for getting server configuration
pub async fn handle_get_configuration(
    app_state: AppState,
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigGetError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;
    
    // Read configuration file
    match fs::read_to_string(&config_path).await {
        Ok(config_json) => {
            let config: Configuration = serde_json::from_str(&config_json)
                .map_err(|e| {
                    eprintln!("Failed to parse configuration JSON: {}", e);
                    warp::reject::custom(ConfigGetError)
                })?;
            
            // Return only server configuration (id, name) - no password
            let config_response = serde_json::json!({
                "config": {
                    "id": config.id,
                    "name": config.name
                }
            });
            
            Ok(warp::reply::with_status(
                warp::reply::json(&config_response),
                warp::http::StatusCode::OK,
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
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigSaveError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // First, save the server configuration (id, name, password) to config.json
    let final_config = Configuration {
        id: Uuid::new_v4().to_string(),
        name: incoming_config.name,
        password: hash_password(&incoming_config.password)
            .map_err(|e| {
                eprintln!("Failed to hash password: {}", e);
                warp::reject::custom(ConfigSaveError)
            })?,
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
    
    println!("Server configuration saved successfully to: {:?}", config_path);
    
    // Now add each profile using the existing handle_create_profile function
    for profile in incoming_config.profiles {
        let profile_request = IncomingProfile {
            id: None,
            name: profile.name,
            color: profile.color,
        };
        
        // Call the existing profile creation handler
        profiles::handle_create_profile(app_state.clone(), profile_request).await
            .map_err(|e| {
                eprintln!("Failed to create profile: {:?}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
    }
    
    // Now add each index using the existing handle_create_local_index function
    for index in incoming_config.indexes {
        let index_request = IncomingMediaIndex {
            id: None,
            name: index.name,
            r#type: index.r#type,
            icon: index.icon,
            custom_icon_file: index.custom_icon_file,
            folders: index.folders,
        };
        
        // Call the existing index creation handler
        indexes::handle_create_local_index(app_state.clone(), index_request).await
            .map_err(|e| {
                eprintln!("Failed to create index: {:?}", e);
                warp::reject::custom(ConfigSaveError)
            })?;
    }
    
    // Fetch the final state from database for response
    let profiles_repo = ProfilesRepo::new(app_state.db_pool.clone());
    let indexes_repo = IndexesRepo::new(app_state.db_pool.clone());
    
    let profiles = profiles_repo.get_all_profiles().await
        .map_err(|e| {
            eprintln!("Failed to fetch profiles for response: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?
        .into_iter()
        .map(ProfileResponse::from)
        .collect();
    
    let indexes = indexes_repo.get_all_indexes().await
        .map_err(|e| {
            eprintln!("Failed to fetch indexes for response: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?
        .into_iter()
        .map(IndexResponse::from)
        .collect();
    
    // Convert to response format (excluding password)
    let config_response = DatabaseConfigurationResponse {
        id: final_config.id,
        name: final_config.name,
        profiles,
        indexes,
    };
    
    println!("Configuration saved successfully with {} profiles and {} indexes", 
             config_response.profiles.len(), config_response.indexes.len());
    
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
) -> Result<impl warp::reply::Reply, warp::Rejection> {
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
) -> Result<impl warp::reply::Reply, warp::Rejection> {
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

// Handler for serving custom icons by index ID
pub async fn handle_get_index_icon(
    app_state: AppState,
    index_id: String,
) -> Result<Box<dyn warp::reply::Reply>, warp::reject::Rejection> {
    // Validate index ID
    if index_id.trim().is_empty() {
        return Ok(Box::new(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "error": "Index ID is required"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        )));
    }

    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| warp::reject::custom(ConfigGetError))?;
    
    // Get the icons directory using OS app data directory
    let icons_dir = crate::config::icons_dir(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get icons directory: {}", e);
            warp::reject::custom(ConfigGetError)
        })?;

    // Try to find the icon file with various extensions
    let icon_extensions = ["png", "jpg", "jpeg", "gif", "bmp", "webp"];
    
    for ext in &icon_extensions {
        let icon_filename = format!("index_{}.{}", index_id, ext);
        let icon_path = icons_dir.join(&icon_filename);
        
        if icon_path.exists() {
            match tokio::fs::read(&icon_path).await {
                Ok(icon_data) => {
                    // Determine content type based on extension
                    let content_type = match *ext {
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "bmp" => "image/bmp",
                        "webp" => "image/webp",
                        _ => "application/octet-stream",
                    };
                    
                    return Ok(Box::new(warp::reply::with_header(
                        warp::reply::with_status(
                            warp::reply::with_header(
                                icon_data,
                                "Content-Type",
                                content_type,
                            ),
                            warp::http::StatusCode::OK,
                        ),
                        "Cache-Control",
                        "public, max-age=31536000", // Cache for 1 year
                    )));
                }
                Err(e) => {
                    eprintln!("Failed to read icon file {:?}: {}", icon_path, e);
                }
            }
        }
    }
    
    // Icon not found
    Ok(Box::new(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "error": "Icon not found"
        })),
        warp::http::StatusCode::NOT_FOUND,
    )))
}

// Helper function to hash passwords
fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}
