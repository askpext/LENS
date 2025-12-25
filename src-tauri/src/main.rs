// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ffmpeg;

use tauri::Manager;

#[tauri::command]
async fn analyze_video(path: String) -> Result<serde_json::Value, String> {
    ffmpeg::get_video_info(&path).await
}

#[tauri::command]
async fn check_ffmpeg_status() -> Result<String, String> {
    ffmpeg::check_and_install().await
}

#[tauri::command]
async fn open_file_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    
    let file = app.dialog()
        .file()
        .add_filter("Video Files", &["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg"])
        .blocking_pick_file();
    
    match file {
        Some(f) => {
            match f.into_path() {
                Ok(path) => Ok(Some(path.to_string_lossy().to_string())),
                Err(e) => Err(format!("Failed to get path: {}", e))
            }
        },
        None => Ok(None)
    }
}

#[tauri::command]
async fn save_json(data: String, app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_dialog::DialogExt;
    
    let file = app.dialog()
        .file()
        .add_filter("JSON", &["json"])
        .set_file_name("video-analysis.json")
        .blocking_save_file();
    
    if let Some(path) = file {
        let path = path.into_path().map_err(|e| e.to_string())?;
        std::fs::write(&path, data).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Ok(())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            analyze_video, 
            check_ffmpeg_status, 
            open_file_dialog,
            save_json  // ADD THIS LINE
        ])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}