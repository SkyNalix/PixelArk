mod gallery;
use gallery::load_images_from_directory;
use gallery::count_elements_in_dir;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_images_from_directory, count_elements_in_dir])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
