use crate::api::state::AppState;
use tauri::WindowBuilder;
use tauri_plugin_dialog::DialogExt;

// Filter out child folders to avoid redundancy
fn filter_child_folders(folders: Vec<String>) -> Vec<String> {
    if folders.is_empty() {
        return folders;
    }
    
    // Sort folders by path length (shorter paths first)
    let mut sorted_folders = folders;
    sorted_folders.sort_by(|a, b| a.len().cmp(&b.len()));
    
    let mut filtered: Vec<String> = Vec::new();
    
    for folder in sorted_folders {
        // Check if this folder is a child of any already filtered folder
        let is_child = filtered.iter().any(|parent| {
            // Ensure the folder starts with parent path and has a path separator after it
            folder.starts_with(parent) && 
            folder.len() > parent.len() && 
            (folder.chars().nth(parent.len()) == Some('/') || folder.chars().nth(parent.len()) == Some('\\'))
        });
        
        if !is_child {
            filtered.push(folder);
        }
    }
    
    filtered
}

// Handler for folder selection endpoint
pub async fn handle_select_folders(app_state: AppState) -> Result<impl warp::Reply, warp::Rejection> {
    let state = app_state.lock().await;
    let app_handle = state.as_ref().ok_or_else(|| warp::reject::custom(FolderSelectionError))?;
    
    // Use Tauri's folder selection dialog
    let folders = select_folders(app_handle.clone()).await
        .map_err(|_| warp::reject::custom(FolderSelectionError))?;
    
    // Filter out child folders
    let filtered_folders = filter_child_folders(folders);
    
    Ok(warp::reply::json(&serde_json::json!({
        "folders": filtered_folders
    })))
}

// Tauri command for folder selection
#[tauri::command]
pub async fn select_folders(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    // Generate a unique window label to avoid conflicts
    let window_label = format!("folder-picker-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
    
    // Create a small invisible window for better dialog positioning
    let temp_window = WindowBuilder::new(&app, &window_label)
        .title("Folder Picker")
        .inner_size(1.0, 1.0)
        .visible(false)
        .build()
        .map_err(|e| format!("Failed to create temp window: {}", e))?;
    
    // Center the window on screen
    if let Err(e) = temp_window.center() {
        println!("Warning: Could not center temp window: {}", e);
    }
    
    // Make the window visible and bring to front
    let _ = temp_window.show();
    let _ = temp_window.set_focus();
    let _ = temp_window.hide();
    
    // Clone the window handle and app handle for use in the blocking thread
    let window_handle = temp_window.clone();
    let app_clone = app.clone();
    
    // Use blocking dialog in a separate thread to avoid hanging
    let result = tokio::task::spawn_blocking(move || {
        // Use the temporary window as parent for better positioning
        let dialog = app_clone.dialog().file().set_parent(&window_handle);
        dialog.blocking_pick_folders()
    }).await.map_err(|e| format!("Task failed: {}", e))?;
    
    // Hide the temporary window but keep it alive
    let _ = temp_window.hide();
    
    // Return the result
    match result {
        Some(paths) => {
            let folder_paths: Vec<String> = paths.into_iter().map(|p| p.to_string()).collect();
            let filtered_paths = filter_child_folders(folder_paths);
            Ok(filtered_paths)
        },
        None => Ok(vec![]), // User cancelled
    }
}

// Custom error type for folder selection
#[derive(Debug)]
struct FolderSelectionError;

impl warp::reject::Reject for FolderSelectionError {}
