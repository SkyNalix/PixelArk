
use std::fs;
use std::path::PathBuf;
use base64::{encode};
use std::fs::File;
use std::io::Read;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn load_images_from_directory(directory: String) -> Vec<String> {
    let path = PathBuf::from(directory);
    let mut images = Vec::new();

    if path.exists() && path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if let Some(ext) = file_path.extension() {
                    if ext == "jpg" || ext == "png" {
                        if let Ok(mut file) = File::open(&file_path) {
                            let mut buffer = Vec::new();
                            if file.read_to_end(&mut buffer).is_ok() {
                                let base64_string = format!("data:image/{};base64,{}", ext.to_string_lossy(), encode(&buffer));
                                images.push(base64_string);
                            }
                        }
                    }
                }
            }
        }
    }
    images
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, load_images_from_directory])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
