#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use index_media_server_lib::{AppState, ExtendedAppState, ServerState, DEFAULT_HTTP_PORT, find_available_port, handle_select_folders, handle_static_file, handle_save_configuration, handle_get_configuration, handle_ping, handle_connect_code, start_https_server, IncomingConfiguration};

use tauri::{
  menu::{Menu, MenuItem},
  tray::TrayIconBuilder,
  include_image, // <-- macro
};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;



fn main() {
  // Create shared state for HTTP server
  let app_state: AppState = Arc::new(Mutex::new(None));
  let app_state_clone = app_state.clone();
  
  // Create extended state for HTTPS port sharing
  let extended_state: ExtendedAppState = Arc::new(Mutex::new(ServerState::new()));
  let extended_state_clone = extended_state.clone();

  tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![index_media_server_lib::select_folders])
    .setup(move |app| {
      // Store app handle in shared state
      {
        let mut state = app_state_clone.blocking_lock();
        *state = Some(app.handle().clone());
      }

      // Find an available HTTP port
      let http_port = match find_available_port(DEFAULT_HTTP_PORT) {
        Ok(port) => port,
        Err(e) => {
          eprintln!("Failed to find available HTTP port: {}", e);
          DEFAULT_HTTP_PORT // fallback to default
        }
      };

      // Start HTTP server for browser communication and static file serving
      let app_state_http = app_state_clone.clone();
      let app_state_http2 = app_state_clone.clone();
      let app_state_http3 = app_state_clone.clone();
      let extended_state_http = extended_state_clone.clone();
      tauri::async_runtime::spawn(async move {
        // API routes
        let select_folders = warp::path("api")
          .and(warp::path("select-folders"))
          .and(warp::post())
          .and(warp::any().map(move || app_state_http.clone()))
          .and_then(handle_select_folders);

        let get_configuration = warp::path("api")
          .and(warp::path("config"))
          .and(warp::get())
          .and(warp::any().map(move || app_state_http3.clone()))
          .and_then(handle_get_configuration);

        let save_configuration = warp::path("api")
          .and(warp::path("config"))
          .and(warp::post())
          .and(warp::body::json())
          .and(warp::any().map(move || app_state_http2.clone()))
          .and_then(|config: IncomingConfiguration, app_state: AppState| handle_save_configuration(app_state, config));


        let ping = warp::path("api")
          .and(warp::path("ping"))
          .and(warp::get())
          .and_then(handle_ping);

        let connect_code = warp::path("api")
          .and(warp::path("connect-code"))
          .and(warp::get())
          .and(warp::any().map(move || extended_state_http.clone()))
          .and_then(handle_connect_code);

        // Static file serving with SPA fallback
        let static_files = warp::path::full()
          .and_then(handle_static_file);

        // Combine routes
        let routes = select_folders
          .or(get_configuration)
          .or(save_configuration)
          .or(ping)
          .or(connect_code)
          .or(static_files)
          .with(warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "OPTIONS"]));

        println!("🚀 Index Media Server running on http://localhost:{}", http_port);
        warp::serve(routes)
          .run(([127, 0, 0, 1], http_port))
          .await;
      });

      // Start HTTPS server for network access
      let extended_state_https = extended_state_clone.clone();
      tauri::async_runtime::spawn(async move {
        match start_https_server(extended_state_https).await {
          Ok(port) => {
            println!("✅ HTTPS server started successfully on port {}", port);
          }
          Err(e) => {
            eprintln!("Failed to start HTTPS server: {}", e);
          }
        }
      });
      // Hide Dock icon as we won't have windows
      #[cfg(target_os = "macos")]
      app.set_activation_policy(tauri::ActivationPolicy::Accessory);

      // Open web interface in browser on startup (only in production)
      #[cfg(not(debug_assertions))]
      {
        let _ = app.opener().open_url(&format!("http://localhost:{}", http_port), None::<&str>);
      }

      // tray icon setup
      let open_i = MenuItem::with_id(app, "open", "Open Index Media Server", true, None::<&str>)?;
      let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

      // Compile-time embed of PNG/ICO relative to src-tauri/Cargo.toml:
      let tray_img = include_image!("icons/tray.png");

      TrayIconBuilder::new()
        .icon(tray_img.clone())
        .icon_as_template(true) // macOS: treat as template for dark/light menu bar
        .menu(&menu)
        .on_menu_event(move |app, e| match e.id.as_ref() {
          "open" => { let _ = app.opener().open_url(&format!("http://localhost:{}", http_port), None::<&str>); }
          "quit" => {
            // Show confirmation dialog before quitting
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
              let app_handle_clone = app_handle.clone();
              app_handle.dialog()
                .message("Are you sure you want to quit Index Media Server? You won't be able to access it again until you restart the app.")
                .title("Quit Index Media Server")
                .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancelCustom("Quit".to_string(), "Cancel".to_string()))
                .show(move |confirmed| {
                  if confirmed {
                    app_handle_clone.exit(0);
                  }
                });
            });
          }
          _ => {}
        })
        .build(app)?;

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("run failed");
}