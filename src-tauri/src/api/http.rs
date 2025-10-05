use warp::Filter;
use crate::api::state::{AppState, ExtendedAppState};
use crate::models::config::IncomingConfiguration;
use crate::api::folders::handle_select_folders;
use crate::api::config::{handle_get_configuration, handle_save_configuration, handle_update_server_password, handle_update_server_name};
use crate::api::handlers::{handle_ping, handle_connect_code, handle_static_file};
use crate::models::config::{ServerPasswordUpdate, ServerNameUpdate};

/// Start the HTTP server for browser communication and static file serving
pub async fn start_http_server(
    http_port: u16,
    app_state: AppState,
    extended_state: ExtendedAppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Clone states for each route
    let app_state_select = app_state.clone();
    let app_state_get_config = app_state.clone();
    let app_state_save_config = app_state.clone();
    let app_state_update_password = app_state.clone();
    let app_state_update_name = app_state.clone();
    let extended_state_connect = extended_state.clone();

    // API routes
    let select_folders = warp::path("api")
        .and(warp::path("select-folders"))
        .and(warp::post())
        .and(warp::any().map(move || app_state_select.clone()))
        .and_then(handle_select_folders);

    let get_configuration = warp::path("api")
        .and(warp::path("config"))
        .and(warp::get())
        .and(warp::any().map(move || app_state_get_config.clone()))
        .and_then(handle_get_configuration);

    let save_configuration = warp::path("api")
        .and(warp::path("config"))
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_save_config.clone()))
        .and_then(|config: IncomingConfiguration, app_state: AppState| handle_save_configuration(app_state, config));

    let ping = warp::path("api")
        .and(warp::path("ping"))
        .and(warp::get())
        .and_then(handle_ping);

    let connect_code = warp::path("api")
        .and(warp::path("connect-code"))
        .and(warp::get())
        .and(warp::any().map(move || extended_state_connect.clone()))
        .and_then(handle_connect_code);

    let update_password = warp::path("api")
        .and(warp::path("server"))
        .and(warp::path("password"))
        .and(warp::put())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_update_password.clone()))
        .and_then(|password_update: ServerPasswordUpdate, app_state: AppState| handle_update_server_password(app_state, password_update));

    let update_name = warp::path("api")
        .and(warp::path("server"))
        .and(warp::path("name"))
        .and(warp::put())
        .and(warp::body::json())
        .and(warp::any().map(move || app_state_update_name.clone()))
        .and_then(|name_update: ServerNameUpdate, app_state: AppState| handle_update_server_name(app_state, name_update));

    // Static file serving with SPA fallback
    let static_files = warp::path::full()
        .and_then(handle_static_file);

    // Combine routes
    let routes = select_folders
        .or(get_configuration)
        .or(save_configuration)
        .or(ping)
        .or(connect_code)
        .or(update_password)
        .or(update_name)
        .or(static_files)
        .with(warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "PUT", "OPTIONS"]));

    println!("🚀 Index Media Server running on http://localhost:{}", http_port);
    warp::serve(routes)
        .run(([127, 0, 0, 1], http_port))
        .await;

    Ok(())
}
