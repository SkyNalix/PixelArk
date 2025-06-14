use std::collections::HashSet;
use natord::compare;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Instant;
use image::ImageFormat;
use tauri::State;
use crate::{get_project_path, ProjectPath};
use crate::cache::{load_cache_set, find_cached_file};
use crate::media_io::{cache_media_thumbnail, MediaFormat};

#[derive(Serialize)]
pub struct ImageElementData {
    index: u32,
    name: String,
    path: String,
    thumbnail_path: String,
    width: u32,
    height: u32,
    size: u64, // in kilobytes
}

#[derive(Serialize)]
pub struct LoadImagesResponse {
    medias: Vec<ImageElementData>,
    no_more_batches: bool,
}

pub struct MediaElement {
    pub full_path: PathBuf,
    pub media_type: MediaFormat,
    pub media_name: String,
    width: u32,
    height: u32,
    size: u64, // in kilobytes
}

fn path_to_media_element(path: PathBuf) -> Option<MediaElement> {
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to open file {:?}: {}", &path, e);
            return None;
        }
    };

    let reader = match image::ImageReader::new(std::io::BufReader::new(&file)).with_guessed_format() {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to guess image format for {}: {:?}", path.display(), e);
            return None;
        }
    };
    let media_type = match reader.format() {
        None => return None,
        Some(ImageFormat::Jpeg) => MediaFormat::JPG,
        Some(ImageFormat::Png) => MediaFormat::PNG,
        Some(ImageFormat::WebP) => MediaFormat::WEBP,
        _ => {
            log::warn!("Unknown image format in file {:?}", path.display());
            return None
        },
    };

    let (width, height) = match reader.into_dimensions() {
        Ok(dimensions) => dimensions,
        Err(e) => {
            log::error!("Failed to get image dimensions of {}: {:?}", path.display(), e);
            return None;
        }
    };

    let media_name = match path.file_stem().and_then(|n| n.to_str()) {
        Some(name) => name.to_string(),
        None => {
            log::error!("Failed to retrieve file's name from the image's full path: {:?}", path);
            return None;
        },
    };

    let size = match file.metadata() {
        Ok(metadata) => metadata.len() / 1000,
        Err(e) => {
            log::error!("Failed to get file size of {}: {:?}", path.display(), e);
            return None;
        }
    };

    Some(MediaElement {
        full_path: path,
        media_type,
        media_name,
        width,
        height,
        size
    })
}

#[tauri::command(async)]
pub fn load_images_from_directory(directory: String, start: i32, stop: i32, state: State<ProjectPath>) -> Result<LoadImagesResponse, String> {
    // debug variables
    let timer = Instant::now();
    let mut cached_images_counter = 0;

    let disable_cache = true;
    let asset_prefix = "http://asset.localhost/";

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
    let mut media_files: Vec<MediaElement> = read_dir
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().is_file() => path_to_media_element(entry.path()),
            Err(e) => {
                log::error!("Failed to read directory entry: {}", e); 
                None
            },
            _ => None
        })
        .collect();

    // Sort filenames
    media_files.sort_by(|a, b| compare(&a.media_name, &b.media_name));

    if range_start >= media_files.len() {
        return Ok(LoadImagesResponse { medias: images, no_more_batches: true}); // Return empty if the range is out of bounds
    }

    // load cached images if caching enabled and cache directory exists
    let cache_directory = root_path.join(".cache").join(&directory);
    let cached_thumbnails: HashSet<PathBuf>;
    if !cache_directory.exists() {
        if let Err(e) = fs::create_dir_all(&cache_directory) {
            log::error!("Failed to create cache directory: {:?}", e);
        }
        cached_thumbnails = HashSet::new()
    } else {
        cached_thumbnails = load_cache_set(&cache_directory);
    }
    
    for (index, media_element) in media_files
        .iter()
        .enumerate()
        .skip(range_start)
        .take(range_stop.saturating_sub(range_start))
    {

        let is_too_small = media_element.size < 1000;

        let mut thumbnail_path: Option<PathBuf> = if is_too_small {Some(media_element.full_path.clone())} else {None};
        
        // try to load cached a thumbnail if it exists
        if !disable_cache && thumbnail_path == None  {
            if let Some(path) = find_cached_file(media_element.media_name.as_str(), &cached_thumbnails) {
                thumbnail_path = Some(path.to_owned());
            }
        }
        
        // if the cached thumbnail doesn't exist, try to create a new thumbnail image and cache it
        if thumbnail_path == None {
            cached_images_counter += 1;
            match cache_media_thumbnail(&media_element, &cache_directory) {
                Ok(path) => {
                    thumbnail_path = Some(path);
                }
                Err(e) => {
                    log::error!("Failed to cache image {}: {:?}", media_element.full_path.display(), e);
                }
            }
        }

        // if the thumbnail creation failed, fallback to the full image instead of a thumbnail
        if thumbnail_path == None {
            thumbnail_path = Some(media_element.full_path.clone())
        }

        let path = format!("{}{}", asset_prefix, media_element.full_path.display());
        let thumbnail_path = format!("{}{}", asset_prefix, thumbnail_path.unwrap().display());
        images.push(ImageElementData {
            index: index as u32,
            name: media_element.media_name.clone(),
            path,
            thumbnail_path,
            width: media_element.width,
            height: media_element.height,
            size: media_element.size,
        });
    }

    log::info!("Amount of cached images: {:?}. Total batch load time: {:?}", cached_images_counter, timer.elapsed());
    Ok(LoadImagesResponse { medias: images, no_more_batches: range_stop >= media_files.len() })
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