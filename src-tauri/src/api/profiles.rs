use crate::models::config::IncomingProfile;
use crate::api::responses::ProfileResponse;
use crate::db::repos::ProfilesRepo;
use crate::api::state::AppState;
use crate::config::config_path;
use tokio::fs;
use warp::reject::custom;

// Custom error types for profile operations
#[derive(Debug)]
pub struct ProfileError;

impl warp::reject::Reject for ProfileError {}

// Handler for getting all profiles
pub async fn handle_get_profiles(
    app_state: AppState,
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Get all profiles from database
    let profiles_repo = ProfilesRepo::new(app_state.db_pool.clone());
    
    let profiles: Vec<ProfileResponse> = profiles_repo.get_all_profiles().await
        .map_err(|e| {
            eprintln!("Failed to fetch profiles: {}", e);
            custom(ProfileError)
        })?
        .into_iter()
        .map(ProfileResponse::from)
        .collect();
    
    let response = serde_json::json!({
        "profiles": profiles
    });
    
    Ok(warp::reply::with_status(
        warp::reply::json(&response),
        warp::http::StatusCode::OK,
    ))
}

// Handler for creating a new profile
pub async fn handle_create_profile(
    app_state: AppState,
    profile_request: IncomingProfile,
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Validate profile name is not empty
    if profile_request.name.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Profile name is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Validate profile color is not empty
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
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| custom(ProfileError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            custom(ProfileError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            custom(ProfileError)
        })?;
    
    let _config: crate::models::config::Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            custom(ProfileError)
        })?;
    
    // Create profile in database
    let profiles_repo = ProfilesRepo::new(app_state.db_pool.clone());
    let profile_id = profiles_repo.add_profile(
        profile_request.name.trim().to_string(),
        profile_request.color.trim().to_string(),
    ).await
        .map_err(|e| {
            eprintln!("Failed to create profile: {}", e);
            custom(ProfileError)
        })?;
    
    // Get the created profile to return in response
    let created_profile = profiles_repo.get_profile_by_id(profile_id).await
        .map_err(|e| {
            eprintln!("Failed to fetch created profile: {}", e);
            custom(ProfileError)
        })?
        .ok_or_else(|| {
            eprintln!("Created profile not found");
            custom(ProfileError)
        })?;
    
    println!("Profile '{}' created successfully with ID: {}", created_profile.name, created_profile.id);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Profile created successfully",
            "profile": ProfileResponse::from(created_profile)
        })),
        warp::http::StatusCode::CREATED,
    ))
}

// Handler for updating a profile
pub async fn handle_update_profile(
    app_state: AppState,
    profile_id: String,
    profile_request: IncomingProfile,
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Validate profile name is not empty
    if profile_request.name.trim().is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Profile name is required and cannot be empty"
            })),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Validate profile color is not empty
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
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| custom(ProfileError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            custom(ProfileError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            custom(ProfileError)
        })?;
    
    let _config: crate::models::config::Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            custom(ProfileError)
        })?;
    
    // Update profile in database
    let profiles_repo = ProfilesRepo::new(app_state.db_pool.clone());
    
    // Parse profile_id as i64
    let profile_id = profile_id.parse::<i64>()
        .map_err(|_| {
            eprintln!("Invalid profile ID format");
            custom(ProfileError)
        })?;
    
    // Check if profile exists
    let _existing_profile = profiles_repo.get_profile_by_id(profile_id).await
        .map_err(|e| {
            eprintln!("Failed to fetch profile: {}", e);
            custom(ProfileError)
        })?
        .ok_or_else(|| {
            eprintln!("Profile not found");
            custom(ProfileError)
        })?;
    
    // Update the profile
    profiles_repo.update_profile(
        profile_id,
        profile_request.name.trim().to_string(),
        profile_request.color.trim().to_string(),
    ).await
        .map_err(|e| {
            eprintln!("Failed to update profile: {}", e);
            custom(ProfileError)
        })?;
    
    // Get the updated profile to return in response
    let updated_profile = profiles_repo.get_profile_by_id(profile_id).await
        .map_err(|e| {
            eprintln!("Failed to fetch updated profile: {}", e);
            custom(ProfileError)
        })?
        .ok_or_else(|| {
            eprintln!("Updated profile not found");
            custom(ProfileError)
        })?;
    
    println!("Profile '{}' updated successfully", updated_profile.name);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Profile updated successfully",
            "profile": ProfileResponse::from(updated_profile)
        })),
        warp::http::StatusCode::OK,
    ))
}

// Handler for deleting a profile
pub async fn handle_delete_profile(
    app_state: AppState,
    profile_id: String,
) -> Result<impl warp::reply::Reply, warp::Rejection> {
    // Get the app handle
    let app_handle_guard = app_state.app_handle.lock().await;
    let app_handle = app_handle_guard.as_ref().ok_or_else(|| custom(ProfileError))?;
    
    // Get the config file path using OS app data directory
    let config_path = config_path(app_handle)
        .map_err(|e| {
            eprintln!("Failed to get config path: {}", e);
            custom(ProfileError)
        })?;
    
    // Read existing configuration
    let config_json = fs::read_to_string(&config_path).await
        .map_err(|e| {
            eprintln!("Failed to read configuration file: {}", e);
            custom(ProfileError)
        })?;
    
    let _config: crate::models::config::Configuration = serde_json::from_str(&config_json)
        .map_err(|e| {
            eprintln!("Failed to parse configuration JSON: {}", e);
            custom(ProfileError)
        })?;
    
    // Delete profile from database
    let profiles_repo = ProfilesRepo::new(app_state.db_pool.clone());
    
    // Parse profile_id as i64
    let profile_id = profile_id.parse::<i64>()
        .map_err(|_| {
            eprintln!("Invalid profile ID format");
            custom(ProfileError)
        })?;
    
    // Check if profile exists before deleting
    let existing_profile = profiles_repo.get_profile_by_id(profile_id).await
        .map_err(|e| {
            eprintln!("Failed to fetch profile: {}", e);
            custom(ProfileError)
        })?
        .ok_or_else(|| {
            eprintln!("Profile not found");
            custom(ProfileError)
        })?;
    
    // Delete the profile
    profiles_repo.delete_profile(profile_id).await
        .map_err(|e| {
            eprintln!("Failed to delete profile: {}", e);
            custom(ProfileError)
        })?;
    
    println!("Profile '{}' deleted successfully", existing_profile.name);
    
    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "success": true,
            "message": "Profile deleted successfully"
        })),
        warp::http::StatusCode::OK,
    ))
}
