use crate::models::config::{Configuration, IncomingConfiguration, MediaIndex};
use crate::utils::image::detect_image_extension;
use crate::api::state::AppState;
use base64::{Engine as _, engine::general_purpose};
use std::env;
use tokio::fs;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{rand_core::OsRng, SaltString};

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
pub async fn handle_get_configuration(_app_state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    // Get the data directory path
    let data_dir = env::current_dir()
        .map_err(|e| {
            eprintln!("Failed to get current directory: {}", e);
            warp::reject::custom(ConfigGetError)
        })?
        .join("data");
    
    let config_path = data_dir.join("config.json");
    
    // Try to read the configuration file
    match fs::read_to_string(&config_path).await {
        Ok(config_json) => {
            // Parse the JSON to validate it's valid
            match serde_json::from_str::<Configuration>(&config_json) {
                Ok(config) => Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "config": config
                    })),
                    warp::http::StatusCode::OK,
                )),
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
    _app_state: AppState,
    incoming_config: IncomingConfiguration,
) -> Result<impl warp::Reply, warp::Rejection> {
    // Create data directory structure
    let data_dir = env::current_dir()
        .map_err(|e| {
            eprintln!("Failed to get current directory: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?
        .join("data");
    
    let icons_dir = data_dir.join("icons");
    
    // Create directories if they don't exist
    fs::create_dir_all(&data_dir).await
        .map_err(|e| {
            eprintln!("Failed to create data directory: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    fs::create_dir_all(&icons_dir).await
        .map_err(|e| {
            eprintln!("Failed to create icons directory: {}", e);
            warp::reject::custom(ConfigSaveError)
        })?;
    
    // Process custom icons and convert to final configuration
    let mut final_indexes = Vec::new();
    
    for incoming_index in incoming_config.indexes {
        let mut final_index = MediaIndex {
            id: incoming_index.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            name: incoming_index.name,
            media_type: incoming_index.media_type,
            icon: incoming_index.icon,
            custom_icon_id: None,
            folders: incoming_index.folders,
        };
        
        // Handle custom icon files if present
        if let Some(custom_icon_data) = incoming_index.custom_icon_file {
            // Generate UUID for the icon
            let icon_id = Uuid::new_v4().to_string();
            let icon_id_clone = icon_id.clone();
            
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
            
            // Save with correct extension
            let icon_path = icons_dir.join(format!("{}.{}", icon_id, extension));
            fs::write(&icon_path, icon_data).await
                .map_err(|e| {
                    eprintln!("Failed to save custom icon: {}", e);
                    warp::reject::custom(ConfigSaveError)
                })?;
            
            // Set the icon ID in the final configuration
            final_index.custom_icon_id = Some(icon_id);
            
            println!("Saved custom icon for index '{}' with ID '{}' as {} to: {:?}", 
                     final_index.name, icon_id_clone, extension, icon_path);
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
    let config_path = data_dir.join("config.json");
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
    
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "message": "Configuration saved successfully"
    })))
}

// Custom error types
#[derive(Debug)]
pub struct ConfigGetError;

#[derive(Debug)]
pub struct ConfigNotFoundError;

#[derive(Debug)]
pub struct ConfigSaveError;

impl warp::reject::Reject for ConfigGetError {}
impl warp::reject::Reject for ConfigNotFoundError {}
impl warp::reject::Reject for ConfigSaveError {}
