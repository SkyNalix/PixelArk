use std::fs;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;

#[derive(PartialEq)]
pub enum MediaType {
    JPG,
    PNG
}

pub fn get_media_type(path: &PathBuf) -> Option<MediaType> {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            log::error!("Failed to open file {:?}: {}", path, e);
            return None;
        }
    };

    let mut header = [0u8; 8];
    if let Err(e) = file.read_exact(&mut header) {
        log::error!("Failed to read header of file {:?}: {}", path, e);
        return None;
    }

    if header.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some(MediaType::PNG)
    } else if header.starts_with(&[0xFF, 0xD8]) {
        Some(MediaType::JPG)
    } else {
        log::warn!("Unknown image format in file {:?}", path);
        None
    }
}

fn read_media_file(media_path: &PathBuf) -> Result<Vec<u8>, String>{
    let mut media_file = File::open(media_path).map_err(|e| format!("Failed open image file: {:?}", e.to_string()))?;

    let mut buffer = Vec::new();
    media_file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read image file: {:?}", e.to_string()))?;
    Ok(buffer)
}


// -------------- JPEG --------------

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

fn make_jpeg_thumbnail(media_buffer: Vec<u8>) -> Result<Vec<u8>, String> {
    let image = decompress_and_scale_jpeg_image(media_buffer, turbojpeg::ScalingFactor::ONE_EIGHTH)?;

    let thumbnail_buffer = compress_jpeg_image(&image)?;
    Ok(thumbnail_buffer.to_vec())
}


// -------------- PNG --------------

fn make_png_thumbnail(media_buffer: Vec<u8>) -> Result<Vec<u8>, String> {
    let mut reader = png::Decoder::new(Cursor::new(media_buffer))
        .read_info().map_err(|e| format!("Failed to decode PNG {}", e))?;

    let mut buf = vec![0; reader.output_buffer_size()];
    let image_info = reader.next_frame(&mut buf)
        .map_err(|e| format!("Failed to read PNG information {}", e))?;
    let image_bytes = &buf[..image_info.buffer_size()];

    let pixel_type = {
        let color_type = image_info.color_type;
        let expected_len = match color_type {
            png::ColorType::Rgba => image_info.width as usize * image_info.height as usize * 4,
            png::ColorType::Rgb  => image_info.width as usize * image_info.height as usize * 3,
            _ => {
                return Err("Unsupported color type".into());
            }
        };
        if image_bytes.len() != expected_len {
            return Err(format!("Unexpected buffer size: got {}, expected {}", image_bytes.len(), expected_len).into());
        }
        
        match color_type {
            png::ColorType::Rgba => fast_image_resize::PixelType::U8x4,
            png::ColorType::Rgb => fast_image_resize::PixelType::U8x3,
            _ => return Err("Unsupported color type".into()),
        }
    };

    let source_image = fast_image_resize::images::Image::from_vec_u8(
        image_info.width, 
        image_info.height, 
        image_bytes.to_vec(), 
        pixel_type
    ).map_err(|e| format!("Failed to create fast_image_resize image source: {}", e))?;
    
    let width = image_info.width / 8;
    let height = image_info.height / 8;
    let mut destination_image = fast_image_resize::images::Image::new(
        width,
        height,
        pixel_type,
    );

    let options = fast_image_resize::ResizeOptions {
        algorithm: fast_image_resize::ResizeAlg::Nearest,
        ..Default::default()
    };
    let mut resizer = fast_image_resize::Resizer::new();
    resizer.resize(&source_image, &mut destination_image, &Some(options))
        .map_err(|e| format!("Failed to resize PNG image: {}", e))?;

    
    // saving as JPEG
    let pixels = destination_image.buffer().to_vec();

    let turbo_jpeg_pixel_format = match pixel_type {
        fast_image_resize::PixelType::U8x3 => turbojpeg::PixelFormat::RGB,
        fast_image_resize::PixelType::U8x4 => turbojpeg::PixelFormat::RGBA,
        _ => return Err("Unsupported pixel format for JPEG".into()),
    };

    let turbo_jpeg_image = turbojpeg::Image {
        pixels,
        width: width as usize,
        pitch: width as usize * turbo_jpeg_pixel_format.size(),
        height: height as usize,
        format: turbo_jpeg_pixel_format,
    };

    let jpeg_buf = compress_jpeg_image(&turbo_jpeg_image)?;
    Ok(jpeg_buf.to_vec())
}

pub fn cache_media_thumbnail(media_path: &PathBuf, media_extension: &MediaType, cache_directory: &PathBuf) -> Result<PathBuf, String> {
    let media_buffer = read_media_file(media_path)?;

    let thumbnail_buffer = match media_extension {
        MediaType::JPG => make_jpeg_thumbnail(media_buffer),
        MediaType::PNG => make_png_thumbnail(media_buffer),
    }?;

    let file_name = media_path.file_name().unwrap().to_str().unwrap().to_string();
    let thumbnail_path = cache_directory.join(file_name);
    fs::write(&thumbnail_path, thumbnail_buffer).map_err(|e| format!("Failed to write cache file: {:?}", e))?;
    Ok(thumbnail_path)
}