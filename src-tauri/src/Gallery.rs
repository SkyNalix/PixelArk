use std::collections::HashSet;
use natord::compare;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri::State;
use crate::{get_project_path, ProjectPath};

#[derive(Serialize)]
pub struct ImageElementData {
    index: u32,
    name: String,
    path: String,
    thumbnail_path: String,
    width: u32,
    height: u32,
}

#[derive(PartialEq)]
pub enum MediaType {
    JPG,
    PNG
}

pub fn get_media_type(path: &PathBuf) -> Option<MediaType> {
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_lowercase();
    match extension.as_str() {
        "jpg" | "jpeg" => Some(MediaType::JPG),
        "png" => Some(MediaType::PNG),
        _ => None,
    }
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

fn read_media_file(media_path: &PathBuf) -> Result<Vec<u8>, String>{
    let mut media_file = File::open(media_path).map_err(|e| format!("Failed open image file: {:?}", e.to_string()))?;
        
    let mut buffer = Vec::new();
    media_file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read image file: {:?}", e.to_string()))?;
    Ok(buffer)
}

fn decompress_and_scale_jpeg_image(media_buffer: Vec<u8>, scaling: turbojpeg::ScalingFactor) -> Result<turbojpeg::Image<Vec<u8>>, String>{
    let mut decompressor = turbojpeg::Decompressor::new().map_err(|e| format!("Failed to create decompressor: {:?}", e))?;
    decompressor.set_scaling_factor(scaling).map_err(|e| format!("Failed to set scaling factor: {:?}", e))?;
    
    let scaled_header = decompressor.read_header(&media_buffer).map_err(|e| e.to_string())?.scaled(scaling);
    let (width, height) = (scaled_header.width, scaled_header.height);
    let mut image = turbojpeg::Image {
        pixels: vec![0u8; width * height * 3],
        width,
        pitch: 3 * width, // size of one image row in memory
        height,
        format: turbojpeg::PixelFormat::RGB,
    };
    decompressor.decompress(&media_buffer, image.as_deref_mut()).map_err(|e| format!("Failed to decompress the jpeg image: {:?}", e))?;
    Ok(image)
}

fn compress_jpeg_image(image: &turbojpeg::Image<Vec<u8>>) -> Result<turbojpeg::OutputBuf, String> {
    let mut compressor = turbojpeg::Compressor::new().map_err(|e| format!("Failed to create compressor: {:?}", e))?;
    compressor.set_quality(50).map_err(|e| format!("Failed to set JPEG quality: {:?}", e))?;
    compressor.set_subsamp(turbojpeg::Subsamp::Sub2x2).map_err(|e| format!("Failed to set JPEG subsampling: {:?}", e))?;
    let mut compressed_image_buffer = turbojpeg::OutputBuf::new_owned();
    compressor.compress(image.as_deref(), &mut compressed_image_buffer).map_err(|e| format!("Failed to compress JPEG: {:?}", e))?;
    Ok(compressed_image_buffer)
}

fn cache_media_thumbnail(media_path: &PathBuf, cache_directory: &PathBuf) -> Result<PathBuf, String> {
    let media_buffer = read_media_file(media_path)?;
    let image = decompress_and_scale_jpeg_image(media_buffer, turbojpeg::ScalingFactor::ONE_EIGHTH)?;
    
    let file_name = media_path.file_name().unwrap().to_str().unwrap().to_string();
    let thumbnail_buffer = compress_jpeg_image(&image)?;

    let thumbnail_path = cache_directory.join(file_name);
    fs::write(&thumbnail_path, thumbnail_buffer).map_err(|e| format!("Failed to write cache file: {:?}", e))?;
    Ok(thumbnail_path)
}

#[tauri::command(async)]
pub fn load_images_from_directory(directory: String, start: i32, stop: i32, state: State<ProjectPath>) -> Result<LoadImagesResponse, String> {
    // debug variables
    let timer = Instant::now();
    let mut processed_count = 0;

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
    let mut media_files: Vec<(PathBuf, MediaType)> = read_dir
        .filter_map(|entry| match entry {
            Ok(entry) if entry.path().is_file() => {
                let extension = match get_media_type(&entry.path()) {
                    Some(ext) => ext,
                    None => {
                        log::warn!("Unknown file in the {} directory: {:?}", directory, entry.path());
                        return None;
                    }
                };
                Some((entry.path(), extension))
            },
            Err(e) => {
                log::error!("Failed to read directory entry: {}", e); 
                None
            },
            _ => None
        })
        .collect();

    // Sort filenames
    media_files.sort_by(|(a,_), (b,_)| compare(&a.display().to_string(), &b.display().to_string()));

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
    
    for (index, (media_full_path, media_extension)) in media_files
        .iter()
        .enumerate()
        .skip(range_start)
        .take(range_stop.saturating_sub(range_start))
    {
        let media_name = match media_full_path.file_stem().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => {
                log::error!("Failed to retrieve file's name from the image's full path: {:?}", media_full_path);
                continue;
            },
        };

        let (width, height) = match image::image_dimensions(&media_full_path) {
            Ok(dimensions) => {
                dimensions
            }
            Err(e) => {
                log::error!("Failed to get image dimensions of {}: {:?}", media_full_path.display(), e);
                continue;
            }
        };
        
        let mut thumbnail_path: Option<PathBuf> = None;
        
        // try to load cached a thumbnail if it exists
        if !disable_cache {
            if let Some(path) = find_cached_file(media_name.as_str(), &cached_thumbnails) {
                thumbnail_path = Some(path.to_owned());
            }
        }
        
        // if the cached thumbnail doesn't exist, try to create a new thumbnail image and cache it
        if thumbnail_path == None {
            match cache_media_thumbnail(&media_full_path, &cache_directory) {
                Ok(path) => {
                    thumbnail_path = Some(path);
                }
                Err(e) => {
                    log::error!("Failed to cache image {}: {:?}", media_full_path.display(), e);
                }
            }
        }

        // if the thumbnail creation failed, fallback to the full image instead of a thumbnail
        if thumbnail_path == None {
            thumbnail_path = Some(media_full_path.clone())
        }
        
        let path = format!("{}{}", asset_prefix, media_full_path.display());
        let thumbnail_path = format!("{}{}", asset_prefix, thumbnail_path.unwrap().display());
        images.push(ImageElementData {
            index: index as u32,
            name: media_name,
            path,
            thumbnail_path,
            width,
            height,
        });

        processed_count += 1;
    }

    log::info!("\nProcessed count:       {:?}", processed_count);
    log::info!("Total batch load time: {:?}", timer.elapsed());
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