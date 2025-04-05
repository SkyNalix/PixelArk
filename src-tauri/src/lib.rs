mod gallery;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            gallery::load_images_from_directory, 
            gallery::count_elements_in_dir, 
            gallery::get_folder_names
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
