mod gallery;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            gallery::load_images_from_directory,
            gallery::get_folder_names,
            gallery::get_image_path
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
