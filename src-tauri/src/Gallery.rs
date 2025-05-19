use std::cmp::Ordering::Equal;
use mozjpeg::ColorSpace;
use std::collections::HashSet;
use image::imageops::FilterType;
use image::{ImageReader, ImageResult};
use natord::compare;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Cursor, Error, Write};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use image::codecs::jpeg::JpegEncoder;
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
    let mut media_files: Vec<PathBuf> = read_dir
        .filter_map(|entry| match entry {
            Ok(e) => Some(e.path()),
            Err(_) => None,
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
    media_files.sort_by(|a, b| compare(&a.display().to_string(), &b.display().to_string()));

    if range_start >= media_files.len() {
        return Ok(LoadImagesResponse { medias: images, no_more_batches: true}); // Return empty if the range is out of bounds
    }

    // load cached images if caching enabled and cache directory exists
    let cache_dir = root_path.join(".cache").join(&directory);
    let cached_images: HashSet<PathBuf>;
    if !cache_dir.exists() {
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            log::error!("Failed to create cache directory: {:?}", e);
        }
        cached_images = HashSet::new()
    } else {
        cached_images = load_cache_set(&cache_dir);
    }
    
    // debug timers
    let mut processed_count = 0;
    let mut decode_time = Duration::ZERO;
    let mut read_time = Duration::ZERO;
    let mut compress_time = Duration::ZERO;
    let mut save_time = Duration::ZERO;

    for (index, media_full_path) in media_files
        .iter()
        .enumerate()
        .skip(range_start)
        .take(range_stop.saturating_sub(range_start))
    {
        let mut current_decode_time = Duration::ZERO;
        let mut current_read_time = Duration::ZERO;
        let mut current_compress_time = Duration::ZERO;
        let mut current_save_time = Duration::ZERO;


        let media_file = match File::open(media_full_path) {
            Ok(f) => f,
            Err(e) => {
                log::error!("Failed open image file: {:?}", e);
                continue;
            },
        };
        let media_reader = std::io::BufReader::new(&media_file);

        let media_extension = match media_full_path.extension().and_then(|ext| ext.to_str()) {
            Some(extension) => extension.to_lowercase(),
            None => {
                log::error!("Failed to retrieve file's extension from the image's full path: {:?}", media_full_path);
                continue;
            }
        };

        let media_name = match media_full_path.file_stem().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => {
                log::error!("Failed to retrieve file's name from the image's full path: {:?}", media_full_path);
                continue;
            },
        };

        if !disable_cache {
            if let Some(cache_img_path) = find_cached_file(media_name.as_str(), &cached_images) {
                match image::image_dimensions(&cache_img_path) {
                    Err(e) => {
                        log::error!("Failed to get image dimensions of cached image {}: {:?}", cache_img_path.display(), e);
                    },
                    Ok((width, height)) => {
                        images.push(ImageElementData {
                            index: index as u32,
                            name: media_name,
                            path: format!("{}{}", asset_prefix, media_full_path.display()),
                            thumbnail_path: format!("{}{}", asset_prefix, cache_img_path.display()),
                            width,
                            height,
                        });
                        continue;
                    }
                }
            }
        }




        // reading
        let start = Instant::now();
        let mut buffer = Vec::new();
        if let Err(e) = media_file.take(10 * 1024 * 1024).read_to_end(&mut buffer) {
            log::error!("Failed to read file: {:?}", e);
            continue;
        }
        current_read_time += start.elapsed();




        // decoding
        let start = Instant::now();
        let mut decompressor = match turbojpeg::Decompressor::new() {
            Ok(d) => d,
            Err(e) => {
                log::error!("Failed to create decompressor: {:?}", e);
                continue;
            }
        };
        let scaling = turbojpeg::ScalingFactor::ONE_EIGHTH;
        if let Err(e) = decompressor.set_scaling_factor(scaling) {
            log::error!("Failed to set scaling factor: {:?}", e);
            continue;
        }
        let scaled_header = decompressor.read_header(&buffer).map_err(|e| e.to_string())?.scaled(scaling);
        let (width, height) = (scaled_header.width, scaled_header.height);
        let mut image = turbojpeg::Image {
            pixels: vec![0u8; width * height * 3],
            width,
            pitch: 3 * width, // size of one image row in memory
            height,
            format: turbojpeg::PixelFormat::RGB,
        };
        if let Err(e) = decompressor.decompress(&buffer, image.as_deref_mut()) {
            log::error!("Failed to set scaling factor: {:?}", e);
            continue;
        }
        current_decode_time += start.elapsed();





        // Compressing
        let start = Instant::now();
        let mut compressor = turbojpeg::Compressor::new().map_err(|e| format!("Failed to create compressor: {:?}", e))?;
        compressor.set_quality(50).map_err(|e| format!("Failed to set JPEG quality: {:?}", e))?;
        compressor.set_subsamp(turbojpeg::Subsamp::Sub2x2).map_err(|e| format!("Failed to set JPEG subsampling: {:?}", e))?;
        let mut output_buf = turbojpeg::OutputBuf::new_owned();
        compressor.compress(image.as_deref(), &mut output_buf).map_err(|e| format!("Failed to compress JPEG: {:?}", e))?;
        current_compress_time += start.elapsed();


        
        
        // saving
        let start = Instant::now();
        let thumbnail_path = cache_dir.join(media_full_path.file_name().unwrap().to_str().unwrap().to_string());
        fs::write(&thumbnail_path, &output_buf).map_err(|e| format!("Failed to write cache file: {:?}", e))?;
        current_save_time += start.elapsed();



        let path = format!("{}{}", asset_prefix, media_full_path.display());
        let thumbnail_path = format!("{}{}", asset_prefix, thumbnail_path.display());
        images.push(ImageElementData {
            index: index as u32,
            name: media_name,
            path,
            thumbnail_path,
            width: width as u32,
            height: height as u32,
        });

        processed_count += 1;
        decode_time += current_decode_time;
        read_time += current_read_time;
        save_time += current_save_time;
        compress_time += current_compress_time;
    }

    // Print average times
    if processed_count > 0 {
        log::info!("\nProcessed count:       {:?}", processed_count);
        log::info!("Average decode_time:   {:?}", decode_time / processed_count);
        log::info!("Average read_time:     {:?}", read_time / processed_count);
        log::info!("Average compress_time: {:?}", compress_time / processed_count);
        log::info!("Average save_time:    {:?}", save_time / processed_count);
    }
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