use image::imageops::FilterType;
use image::ImageReader;
use natord::compare;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tauri::State;
use crate::{get_project_path, ProjectPath};

#[derive(Serialize)]
pub struct ImageData {
    name: String,
    path: String,
    width: u32,
    height: u32,
}

fn get_cache(cache_dir: &PathBuf, file_name: &str) -> Option<PathBuf> {
    if !cache_dir.is_dir() {
        return None;
    }
    let path = cache_dir.join(file_name);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[tauri::command]
pub fn load_images_from_directory(directory: String, start: i32, stop: i32, state: State<ProjectPath>) -> Result<Vec<ImageData>, String> {
    let timer = Instant::now();

    let root_path = get_project_path(state);
    if None == root_path {
        return Err("Project path not defined".to_string());
    }
    let root_path = root_path.unwrap();

    let path = root_path.join(&directory);
    let mut images = Vec::new();

    if !path.is_dir() {
        return Err(format!("Invalid directory: {}", directory));
    }

    let read_dir = fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;

    // Collect and filter image files
    let mut image_files: Vec<PathBuf> = read_dir
        .filter_map(|entry| match entry {
            Ok(e) => Some(e.path()),
            Err(_) => None, // You could also log this if needed
        })
        .filter(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|s| {
                    let lower = s.to_lowercase();
                    matches!(lower.as_str(), "jpg" | "jpeg" | "png")
                })
                .unwrap_or(false)
        })
        .collect();

    // Sort filenames
    image_files.sort_by(|a, b| compare(&a.to_string_lossy(), &b.to_string_lossy()));
    println!("{}", " ".repeat(20));

    let range_start = start.max(0) as usize;
    let range_stop = stop.max(0) as usize;

    if range_start >= image_files.len() {
        return Ok(images); // Return empty if range is out of bounds
    }

    let cache_dir = root_path.join(".cache").join(&directory);
    fs::create_dir(&cache_dir).ok(); // Make sure it exists

    // Timers
    let mut read_time = Duration::ZERO;
    let mut open_time = Duration::ZERO;
    let mut decode_time = Duration::ZERO;
    let mut resize_time = Duration::ZERO;
    let mut cache_time = Duration::ZERO;

    let mut processed_count = 0;

    for full_path in image_files
        .iter()
        .skip(range_start)
        .take(range_stop.saturating_sub(range_start))
    {
        let file_path = full_path.strip_prefix(&root_path).ok().unwrap();

        let mut file = match File::open(full_path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let file_name = match full_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        if let Some(cache_img_path) = get_cache(&cache_dir, &file_name) {
            if let Ok(img) = image::open(&cache_img_path) {
                let width = img.width();
                let height = img.height();
                images.push(ImageData {
                    name: file_name,
                    path: file_path.to_string_lossy().to_string(),
                    width,
                    height,
                });
                processed_count += 1;
                continue;
            }
        }

        let start = Instant::now();
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_err() {
            continue;
        }
        read_time += start.elapsed();

        // reading image
        let start = Instant::now();
        let reader = match ImageReader::new(Cursor::new(&buffer)).with_guessed_format() {
            Ok(r) => r,
            Err(_) => continue,
        };
        open_time += start.elapsed();
        let start = Instant::now();
        let img = match reader.decode() {
            Ok(i) => i,
            Err(_) => continue,
        };
        decode_time += start.elapsed();

        // resizing image
        let start = Instant::now();
        let width = img.width();
        let height = img.height();
        let resized = img.resize(
            (width as f32 * 0.2) as u32,
            (height as f32 * 0.2) as u32,
            FilterType::Nearest,
        );
        resize_time = start.elapsed();

        // saving image to cache folder
        let start = Instant::now();
        resized.save(cache_dir.join(&file_name)).ok();
        cache_time += start.elapsed();

        images.push(ImageData {
            name: file_name,
            path: file_path.to_string_lossy().to_string(),
            width,
            height,
        });

        processed_count += 1;
    }

    // Print average times
    if processed_count > 0 {
        println!("Processed count:     {:?}", processed_count);
        println!("Average read_time:   {:?}", read_time / processed_count);
        println!("Average open_time:   {:?}", open_time / processed_count);
        println!("Average decode_time: {:?}", decode_time / processed_count);
        println!("Average resize_time: {:?}", resize_time / processed_count);
        println!("Average cache_time: {:?}", cache_time / processed_count);
    } else {
        println!("No files were processed.");
    }

    println!("Total time: {:?}", timer.elapsed());
    Ok(images)
}

#[tauri::command]
pub fn get_image_path(file_path: String, state: State<ProjectPath>) -> Result<String, String> {
    let root_path = get_project_path(state);
    if None == root_path {
        return Err("Project path not defined".to_string());
    }

    let file_path = root_path.unwrap().join(".cache").join(file_path);
    if file_path.exists() {
        Ok(file_path.to_string_lossy().into())
    } else {
        Err("Image not found".into())
    }
}

#[tauri::command]
pub fn get_folder_names(directory: &str, state: State<ProjectPath>) -> Result<Vec<String>, String> {
    let root_path = get_project_path(state);
    if None == root_path {
        return Err("Project path not defined".to_string());
    }

    let path = root_path.unwrap().join(Path::new(directory));

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", directory));
    }

    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;

    let mut folder_names: Vec<String> = entries
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.path().is_dir() {
                    let folder_name = e.file_name().to_str().map(|s| s.to_string());
                    if folder_name.is_none() {
                        return None
                    }
                    let folder_name = folder_name.unwrap().to_string();
                    if folder_name.eq(".cache") {
                        None
                    } else {
                        Some(folder_name.to_string())
                    }
                } else {
                    None
                }
            })
        })
        .collect();
    folder_names.sort_by(|a, b| compare(&a, &b));

    Ok(folder_names)
}