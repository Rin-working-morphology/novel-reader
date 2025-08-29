use serde::{Deserialize, Serialize};

mod epub;
mod txt;

#[derive(Debug, Serialize, Deserialize)]
struct ReadingProgress {
    folder_path: String,
    current_file: String,
    current_chapter: usize,
    scroll_position: f64,
    last_read_time: String,
}

// 保存阅读进度
#[tauri::command]
async fn save_reading_progress(
    app_handle: tauri::AppHandle,
    progress: ReadingProgress,
) -> Result<(), String> {
    let store = tauri_plugin_store::StoreBuilder::new(&app_handle, "reading_progress.json").build();

    match store {
        Ok(store) => {
            store.set("progress", serde_json::to_value(progress).unwrap());

            if let Err(e) = store.save() {
                return Err(format!("Failed to save store: {}", e));
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to create store: {}", e)),
    }
}

// 加载阅读进度
#[tauri::command]
async fn load_reading_progress(
    app_handle: tauri::AppHandle,
) -> Result<Option<ReadingProgress>, String> {
    let store = tauri_plugin_store::StoreBuilder::new(&app_handle, "reading_progress.json").build();

    match store {
        Ok(store) => {
            if let Err(e) = store.reload() {
                return Ok(None); // 文件不存在时返回None
            }

            match store.get("progress") {
                Some(value) => match serde_json::from_value::<ReadingProgress>(value.clone()) {
                    Ok(progress) => Ok(Some(progress)),
                    Err(_) => Ok(None),
                },
                None => Ok(None),
            }
        }
        Err(_) => Ok(None),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            txt::scan_txt_files,
            txt::read_txt_file,
            txt::parse_chapters,
            epub::load_epub_file,
            epub::get_epub_info,    
            epub::load_epub_chapter,
            save_reading_progress,
            load_reading_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
