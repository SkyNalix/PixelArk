use std::fs;
use std::path::{Path, PathBuf};
use base64::{encode};
use std::fs::File;
use std::io::Read;
use serde::Serialize;
use natord::compare;
use image::io::Reader as ImageReader;
use std::io::Cursor;

#[derive(Serialize)]
pub struct ImageData {
    image_index: i32,
    image_name: String,
    image_base64: String,
    width: u32,
    height: u32,
}

#[tauri::command]
pub fn load_images_from_directory(directory: String, start: i32, stop: i32) -> Vec<ImageData> {
    let path = Path::new(&directory);
    let mut images = Vec::new();

    if !path.exists() || !path.is_dir() {
        return images;
    }

    // Collect all image file paths
    let mut image_files: Vec<PathBuf> = fs::read_dir(path)
        .unwrap_or_else(|_| panic!("Failed to read directory: {}", directory))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            if let Some(ext) = p.extension() {
                matches!(ext.to_str(), Some("jpg" | "jpeg" | "png"))
            } else {
                false
            }
        })
        .collect();

    // Sort filenames naturally (handling numbers correctly)
    image_files.sort_by(|a, b| compare(&a.to_string_lossy(), &b.to_string_lossy()));

    // Read images only in the specified range
    for (index, file_path) in image_files.iter().enumerate().skip(start as usize).take((stop - start) as usize) {
        if let Ok(mut file) = File::open(file_path) {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                if let Some(file_name) = file_path.file_name() {
                    let base64_string = format!(
                        "data:image/{};base64,{}",
                        file_path.extension().unwrap().to_string_lossy(),
                        encode(&buffer)
                    );

                    // Get image dimensions
                    let mut width = 0;
                    let mut height = 0;
                    if let Ok(img) = ImageReader::new(Cursor::new(&buffer)).with_guessed_format().unwrap().decode() {
                        width = img.width();
                        height = img.height();
                    }

                    images.push(ImageData {
                        image_index: index as i32,
                        image_name: file_name.to_string_lossy().to_string(),
                        image_base64: base64_string,
                        width,
                        height,
                    });
                }
            }
        }
    }

    images
}

#[tauri::command]
pub fn count_elements_in_dir(directory: &str) -> i32 {
    let path = Path::new(directory);
    if path.is_dir() {
        match fs::read_dir(path) {
            Ok(entries) => entries.count() as i32,
            Err(_) => -1,
        }
    } else {
        -1
    }
}

#[tauri::command]
pub fn get_folder_names(directory: &str) -> Result<Vec<String>, String> {
    let path = Path::new(directory);
    
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", directory));
    }

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", directory));
    }

    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    
    let mut folder_names: Vec<String> = entries
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.path().is_dir() {
                    e.file_name().to_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
        })
        .collect();
    folder_names.sort_by(|a, b| compare(&a, &b));

    Ok(folder_names)
}