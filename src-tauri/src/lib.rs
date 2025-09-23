use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};
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

#[derive(Debug, Serialize, Deserialize)]
struct AppearanceSettings {
    theme: String,              // "default" 或 "dark"
    show_file_sidebar: bool,    // 是否显示文件列表
    show_outline_sidebar: bool, // 是否显示右侧大纲列表
    outline_collapsed: bool,    // 是否折叠右侧大纲列表
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

// 保存外观设置
#[tauri::command]
async fn save_appearance_settings(
    app_handle: tauri::AppHandle,
    settings: AppearanceSettings,
) -> Result<(), String> {
    let store = tauri_plugin_store::StoreBuilder::new(&app_handle, "appearance_settings.json").build();

    match store {
        Ok(store) => {
            store.set("settings", serde_json::to_value(settings).unwrap());

            if let Err(e) = store.save() {
                return Err(format!("Failed to save store: {}", e));
            }
            Ok(())
        }
        Err(e) => Err(format!("Failed to create store: {}", e)),
    }
}

// 加载外观设置
#[tauri::command]
async fn load_appearance_settings(
    app_handle: tauri::AppHandle,
) -> Result<Option<AppearanceSettings>, String> {
    let store = tauri_plugin_store::StoreBuilder::new(&app_handle, "appearance_settings.json").build();

    match store {
        Ok(store) => {
            if let Err(e) = store.reload() {
                return Ok(None); // 文件不存在时返回None
            }

            match store.get("settings") {
                Some(value) => match serde_json::from_value::<AppearanceSettings>(value.clone()) {
                    Ok(settings) => Ok(Some(settings)),
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
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "关闭", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let tray = TrayIconBuilder::new()
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        // in this example, let's show and focus the main window when the tray is clicked
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        let _ = app.save_window_state(StateFlags::all()); // 保存窗口状态
                        app.exit(0);
                    }
                    _ => {}
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            Ok(())
        })
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
            save_appearance_settings,
            load_appearance_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
