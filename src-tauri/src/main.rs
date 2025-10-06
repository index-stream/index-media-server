#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use index_media_server_lib::{AppState, ExtendedAppState, ServerState, DEFAULT_HTTP_PORT, find_available_port, start_http_server, start_https_server, generate_secure_token};

use tauri::{
  menu::{Menu, MenuItem},
  tray::TrayIconBuilder,
  include_image, // <-- macro
};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;
use std::sync::Arc;
use tokio::sync::Mutex;



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

      // Generate a secure token for local access
      let startup_token = generate_secure_token();

      // Start HTTP server for browser communication and static file serving
      let app_state_http = app_state_clone.clone();
      let extended_state_http = extended_state_clone.clone();
      let startup_token_http = startup_token.clone();
      tauri::async_runtime::spawn(async move {
        if let Err(e) = start_http_server(http_port, app_state_http, extended_state_http, startup_token_http).await {
          eprintln!("Failed to start HTTP server: {}", e);
        }
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
        let url = format!("http://localhost:{}?token={}", http_port, startup_token);
        let _ = app.opener().open_url(&url, None::<&str>);
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
          "open" => { 
            let url = format!("http://localhost:{}?token={}{}", 
              http_port, 
              startup_token,
              if cfg!(debug_assertions) { "&dev=local" } else { "" }
            );
            let _ = app.opener().open_url(&url, None::<&str>); 
          }
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