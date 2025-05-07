use std::path::{Path, PathBuf};
use std::sync::Mutex;
use chrono::Local;
use tauri::State;

mod gallery;

pub struct ProjectPath(Mutex<Option<PathBuf>>);

#[tauri::command]
fn set_project_path(path: String, state: State<ProjectPath>) {
    let mut stored_path = state.0.lock().unwrap();
    *stored_path = Some(Path::new(path.as_str()).to_path_buf());
}

#[tauri::command]
fn get_project_path(state: State<ProjectPath>) -> Option<PathBuf> {
    let stored_path = state.0.lock().unwrap();
    stored_path.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let default_path = Some(Path::new("C:\\dev\\PixelArk\\images100").to_path_buf());

    let logging_plugin = tauri_plugin_log::Builder::new()
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Webview,
        ))
        .format(|out, message, record| {
            let timestamp = Local::now().format("%H:%M:%S").to_string();
            out.finish(format_args!(
                "{} [{}] {}",
                timestamp,
                record.level(),
                message
            ))
        })
        .build();

    tauri::Builder::default()
        .plugin(logging_plugin)
        .manage(ProjectPath(Mutex::new(default_path)))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            set_project_path,
            get_project_path,
            gallery::load_images_from_directory,
            gallery::get_folder_names
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
