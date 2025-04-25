use std::path::{Path, PathBuf};
use std::sync::Mutex;
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

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(ProjectPath(Mutex::new(default_path)))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            set_project_path,
            get_project_path,
            gallery::load_images_from_directory,
            gallery::get_folder_names,
            gallery::get_image_path
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
