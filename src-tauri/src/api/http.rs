use warp::Filter;
use crate::api::state::AppState;
use crate::models::config::IncomingConfiguration;
use crate::api::folders::handle_select_folders;
use crate::api::config::{handle_get_configuration, handle_save_configuration, handle_update_server_password, handle_update_server_name, handle_get_index_icon};
use crate::api::profiles::{handle_get_profiles, handle_create_profile, handle_update_profile, handle_delete_profile};
use crate::api::indexes::{handle_get_indexes, handle_create_local_index, handle_update_index, handle_delete_index, handle_queue_index_scan};
use crate::api::handlers::{handle_ping, handle_connect_code, handle_static_file};
use crate::models::config::{ServerPasswordUpdate, ServerNameUpdate, IncomingProfile, IncomingMediaIndex, IndexUpdateRequest};

/// Start the HTTP server for browser communication and static file serving
pub async fn start_http_server(
    http_port: u16,
    app_state: AppState,
    startup_token: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Clone states for each route
    let app_state_select = app_state.clone();
    let app_state_get_config = app_state.clone();
    let app_state_save_config = app_state.clone();
    let app_state_update_password = app_state.clone();
    let app_state_update_name = app_state.clone();
    let app_state_create_profile = app_state.clone();
    let app_state_update_profile = app_state.clone();
    let app_state_delete_profile = app_state.clone();
    let app_state_create_index = app_state.clone();
    let app_state_update_index = app_state.clone();
    let app_state_delete_index = app_state.clone();

    let app_state_get_profiles = app_state.clone();
    let app_state_get_indexes = app_state.clone();
    let app_state_get_index_icon = app_state.clone();
    let app_state_queue_scan = app_state.clone();

    // Token validation filter for API endpoints
    let token_validation = warp::header::<String>("authorization")
        .and_then(move |auth_header: String| {
            let expected_token = startup_token.clone();
            async move {
                if auth_header.starts_with("Bearer ") {
                    let token = &auth_header[7..]; // Remove "Bearer " prefix
                    if token == expected_token {
                        Ok(())
                    } else {
                        Err(warp::reject::custom(TokenValidationError))
                    }
                } else {
                    Err(warp::reject::custom(TokenValidationError))
                }
            }
        });

    // API routes
    let select_folders = warp::path("api")
        .and(warp::path("select-folders"))
        .and(warp::post())
        .and(token_validation.clone())
        .and(warp::any().map(move || app_state_select.clone()))
        .and_then(|_, app_state: AppState| handle_select_folders(app_state));

    let get_configuration = warp::path("api")
        .and(warp::path("config"))
        .and(warp::get())
        .and(token_validation.clone())
        .and(warp::any().map(move || app_state_get_config.clone()))
        .and_then(|_, app_state: AppState| handle_get_configuration(app_state));

    let save_configuration = warp::path("api")
        .and(warp::path("config"))
        .and(warp::post())
        .and(token_validation.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_save_config.clone()))
        .and_then(|_, config: IncomingConfiguration, app_state: AppState| handle_save_configuration(app_state, config));

    let ping = warp::path("api")
        .and(warp::path("ping"))
        .and(warp::get())
        .and(token_validation.clone())
        .and_then(|_| handle_ping());

    let connect_code = warp::path("api")
        .and(warp::path("connect-code"))
        .and(warp::get())
        .and(token_validation.clone())
        .and(warp::any().map(move || app_state.clone()))
        .and_then(|_, app_state: AppState| handle_connect_code(app_state));

    let update_password = warp::path("api")
        .and(warp::path("server"))
        .and(warp::path("password"))
        .and(warp::put())
        .and(token_validation.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_update_password.clone()))
        .and_then(|_, password_update: ServerPasswordUpdate, app_state: AppState| handle_update_server_password(app_state, password_update));

    let update_name = warp::path("api")
        .and(warp::path("server"))
        .and(warp::path("name"))
        .and(warp::put())
        .and(token_validation.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_update_name.clone()))
        .and_then(|_, name_update: ServerNameUpdate, app_state: AppState| handle_update_server_name(app_state, name_update));

    let create_profile = warp::path("api")
        .and(warp::path("profile"))
        .and(warp::post())
        .and(token_validation.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_create_profile.clone()))
        .and_then(|_, profile_request: IncomingProfile, app_state: AppState| handle_create_profile(app_state, profile_request));

    let update_profile = warp::path("api")
        .and(warp::path("profile"))
        .and(warp::path::param::<String>())
        .and(warp::put())
        .and(token_validation.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_update_profile.clone()))
        .and_then(|profile_id: String, _, profile_request: IncomingProfile, app_state: AppState| handle_update_profile(app_state, profile_id, profile_request));

    let delete_profile = warp::path("api")
        .and(warp::path("profile"))
        .and(warp::path::param::<String>())
        .and(warp::delete())
        .and(token_validation.clone())
        .and(warp::any().map(move || app_state_delete_profile.clone()))
        .and_then(|profile_id: String, _, app_state: AppState| handle_delete_profile(app_state, profile_id));

    let create_local_index = warp::path("api")
        .and(warp::path("index"))
        .and(warp::path("local"))
        .and(warp::post())
        .and(token_validation.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_create_index.clone()))
        .and_then(|_, index_request: IncomingMediaIndex, app_state: AppState| handle_create_local_index(app_state, index_request));

    let update_index = warp::path("api")
        .and(warp::path("index"))
        .and(warp::path::param::<String>())
        .and(warp::put())
        .and(token_validation.clone())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_update_index.clone()))
        .and_then(|index_id: String, _, index_request: IndexUpdateRequest, app_state: AppState| handle_update_index(app_state, index_id, index_request));

    let delete_index = warp::path("api")
        .and(warp::path("index"))
        .and(warp::path::param::<String>())
        .and(warp::delete())
        .and(token_validation.clone())
        .and(warp::any().map(move || app_state_delete_index.clone()))
        .and_then(|index_id: String, _, app_state: AppState| handle_delete_index(app_state, index_id));

    let get_profiles = warp::path("api")
        .and(warp::path("profiles"))
        .and(warp::get())
        .and(token_validation.clone())
        .and(warp::any().map(move || app_state_get_profiles.clone()))
        .and_then(|_, app_state: AppState| handle_get_profiles(app_state));

    let get_indexes = warp::path("api")
        .and(warp::path("indexes"))
        .and(warp::get())
        .and(token_validation.clone())
        .and(warp::any().map(move || app_state_get_indexes.clone()))
        .and_then(|_, app_state: AppState| handle_get_indexes(app_state));

    // Icon serving route (no authorization required for img tags)
    let get_index_icon = warp::path("api")
        .and(warp::path("index"))
        .and(warp::path::param::<String>())
        .and(warp::path("icon"))
        .and(warp::get())
        .and(warp::any().map(move || app_state_get_index_icon.clone()))
        .and_then(|index_id: String, app_state: AppState| handle_get_index_icon(app_state, index_id));

    let queue_index_scan = warp::path("api")
        .and(warp::path("index"))
        .and(warp::path::param::<String>())
        .and(warp::path("scan-job"))
        .and(warp::post())
        .and(token_validation.clone())
        .and(warp::any().map(move || app_state_queue_scan.clone()))
        .and_then(|index_id: String, _, app_state: AppState| handle_queue_index_scan(app_state, index_id));

    // Static file serving with SPA fallback (only for non-API paths)
    let static_files = warp::path::full()
        .and_then(|path: warp::path::FullPath| async move {
            // Don't serve static files for API routes
            if path.as_str().starts_with("/api/") {
                Err(warp::reject::not_found())
            } else {
                handle_static_file(path).await
            }
        });

    // Combine routes
    let routes = select_folders
        .or(get_configuration)
        .or(save_configuration)
        .or(ping)
        .or(connect_code)
        .or(update_password)
        .or(update_name)
        .or(get_profiles)
        .or(create_profile)
        .or(update_profile)
        .or(delete_profile)
        .or(get_indexes)
        .or(create_local_index)
        .or(update_index)
        .or(delete_index)
        .or(get_index_icon)
        .or(queue_index_scan)
        .or(static_files)
        .recover(move |rejection: warp::Rejection| async move {
            if rejection.find::<TokenValidationError>().is_some() {
                Ok(warp::reply::with_status(
                    warp::reply::json(&serde_json::json!({
                        "success": false,
                        "error": "Unauthorized",
                        "message": "Invalid or missing authorization token"
                    })),
                    warp::http::StatusCode::UNAUTHORIZED,
                ))
            } else {
                Err(rejection)
            }
        })
        .with(warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "OPTIONS"]));

    println!("🚀 Index Media Server running on http://localhost:{}", http_port);
    warp::serve(routes)
        .run(([127, 0, 0, 1], http_port))
        .await;

    Ok(())
}

// Custom error type for token validation
#[derive(Debug)]
pub struct TokenValidationError;

impl warp::reject::Reject for TokenValidationError {}
