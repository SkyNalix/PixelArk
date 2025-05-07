use std::collections::HashSet;
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
pub struct ImageElementData {
    name: String,
    path: String,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
pub struct LoadImagesResponse {
    medias: Vec<ImageElementData>,
    no_more_batches: bool,
}

pub fn load_cache_set(cache_dir: &PathBuf) -> HashSet<PathBuf> {
    if !cache_dir.is_dir() {
        log::warn!("Provided cache path is not a directory: {:?}", cache_dir);
        return HashSet::new();
    }

    let entries = fs::read_dir(cache_dir);
    if let Err(e) = entries {
        log::warn!("Failed to read cache directory: {:?}", e);
        return HashSet::new();
    }

    entries.unwrap().filter_map(
        |entry| {
            match entry {
                Err(e) => {
                    log::error!("Failed to read a file entry in {:?}: {}", cache_dir, e);
                    None
                }
                Ok(entry) => {
                   Some(entry.path())
                }
            }
        }
    ).collect()
}

fn find_cached_file<'a>(file_name: &str, cache_set: &'a HashSet<PathBuf>) -> Option<&'a PathBuf> {
    cache_set.iter().find(|p| {
        p.file_name()
            .and_then(|f| f.to_str())
            .map(|f| f == file_name)
            .unwrap_or(false)
    })
}

#[tauri::command(async)]
pub fn load_images_from_directory(directory: String, start: i32, stop: i32, state: State<ProjectPath>) -> Result<LoadImagesResponse, String> {
    let timer = Instant::now();

    let mut images = Vec::new();
    let range_start = start.max(0) as usize;
    let range_stop = stop.max(0) as usize;

    if range_start > range_stop {
        log::error!("Illegal batch loading start {} and stop {} indexes", start, stop);
        return Ok(LoadImagesResponse { medias: images, no_more_batches: false});
    }

    let root_path = match get_project_path(state) {
        Some(path) => path,
        None => return Err("Project path not defined".to_string()),
    };

    let path = root_path.join(&directory);

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

    if range_start >= image_files.len() {
        return Ok(LoadImagesResponse { medias: images, no_more_batches: true}); // Return empty if the range is out of bounds
    }

    let cache_dir = root_path.join(".cache").join(&directory);
    let cached_images;
    if cache_dir.exists() {
        cached_images = load_cache_set(&cache_dir);
    } else {
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            log::error!("Failed to create cache directory: {:?}", e);
        }
        cached_images= HashSet::new()
    }

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

        let mut file = match File::open(full_path) {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed open image file: {:?}", e);
                continue;
            },
        };

        let file_name = match full_path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => {
                log::error!("Failed to retrieve file's name from the image's full path: {:?}", full_path);
                continue;
            },
        };

        if let Some(cache_img_path) = find_cached_file(file_name.as_str(), &cached_images) {
            match image::image_dimensions(&cache_img_path) {
                Err(e) => {
                    log::error!("Failed to get image dimensions of cached image {}: {:?}", cache_img_path.display(), e);
                },
                Ok((width, height)) => {
                    let path = cache_img_path.to_string_lossy().to_string();
                    println!("{}", path);
                    images.push(ImageElementData {
                        name: file_name,
                        path: format!("http://asset.localhost/{}", path),
                        width,
                        height,
                    });
                    continue;
                }
            }
        }

        let start = Instant::now();
        let mut buffer = Vec::new();
        if let Err(e) = file.read_to_end(&mut buffer) {
            log::error!("Failed to read image file: {:?}", e);
            continue;
        }
        read_time += start.elapsed();

        // reading image
        let start = Instant::now();
        let reader = match ImageReader::new(Cursor::new(&buffer)).with_guessed_format() {
            Ok(r) => r,
            Err(e) => {
                log::error!("Failed to create image reader: {:?}", e);
                continue;
            },
        };
        open_time += start.elapsed();
        let start = Instant::now();
        let img = match reader.decode() {
            Ok(i) => i,
            Err(e) => {
                log::error!("Failed to decode image: {:?}", e);
                continue;
            },
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

        // saving image to the cache folder
        let start = Instant::now();
        let cache_path = cache_dir.join(&file_name);
        if let Err(e) = resized.save(&cache_path) {
            log::error!("Failed to save image to cache: {:?}", e);
        }
        cache_time += start.elapsed();

        images.push(ImageElementData {
            name: file_name,
            path: format!("http://asset.localhost/{}", cache_path.to_string_lossy().to_string()),
            width,
            height,
        });

        processed_count += 1;
    }

    // Print average times
    if processed_count > 0 {
        log::info!("\nProcessed count:       {:?}", processed_count);
        log::info!("Average read_time:     {:?}", read_time / processed_count);
        log::info!("Average open_time:     {:?}", open_time / processed_count);
        log::info!("Average decode_time:   {:?}", decode_time / processed_count);
        log::info!("Average resize_time:   {:?}", resize_time / processed_count);
        log::info!("Average cache_time:    {:?}", cache_time / processed_count);
    }
    log::info!("Total batch load time: {:?}", timer.elapsed());
    Ok(LoadImagesResponse { medias: images, no_more_batches: range_stop >= image_files.len() })
}

#[tauri::command]
pub fn get_folder_names(directory: &str, state: State<ProjectPath>) -> Result<Vec<String>, String> {
    let root_path = match get_project_path(state) {
        Some(path) => path,
        None => return Err("Project path not defined".to_string()),
    };

    let path = root_path.join(Path::new(directory));

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