use std::fs;
use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use crate::gallery::MediaElement;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum MediaFormat {
    JPG,
    PNG,
    WEBP
}

fn read_media_file(media_path: &PathBuf) -> Result<Vec<u8>, String>{
    let mut media_file = File::open(media_path).map_err(|e| format!("Failed open image file: {:?}", e.to_string()))?;

    let mut buffer = Vec::new();
    media_file.read_to_end(&mut buffer).map_err(|e| format!("Failed to read image file: {:?}", e.to_string()))?;
    Ok(buffer)
}


pub fn cache_media_thumbnail(media_element: &MediaElement, cache_directory: &PathBuf) -> Result<PathBuf, String> {
    let media_buffer: Vec<u8> = read_media_file(&media_element.full_path)?;

    let thumbnail_buffer: Vec<u8> = match media_element.media_type {
        MediaFormat::JPG => make_jpeg_thumbnail(media_buffer),
        MediaFormat::PNG => make_png_thumbnail(media_buffer),
        MediaFormat::WEBP => make_webp_thumbnail(media_buffer)
    }?;

    let thumbnail_path = cache_directory.join(media_element.media_name.clone() + ".jpg");
    fs::write(&thumbnail_path, thumbnail_buffer).map_err(|e| format!("Failed to write cache file: {:?}", e))?;
    Ok(thumbnail_path)
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

fn resize_pixels_data<F>(pixels_data: Vec<u8>, pixel_type: fast_image_resize::PixelType, origin_width: u32, original_height: u32, scale_dimensions: F ) -> Result<Vec<u8>, String>
    where
        F: Fn(u32, u32) -> (u32, u32),
{
    let source_image = fast_image_resize::images::Image::from_vec_u8(
        origin_width,
        original_height,
        pixels_data,
        pixel_type
    ).map_err(|e| format!("Failed to create fast_image_resize image source: {}", e))?;

    let (scaled_width, scaled_height) = scale_dimensions(origin_width, original_height);
    let mut destination_image = fast_image_resize::images::Image::new(
        scaled_width,
        scaled_height,
        pixel_type,
    );

    let options = fast_image_resize::ResizeOptions {
        algorithm: fast_image_resize::ResizeAlg::Nearest,
        ..Default::default()
    };
    let mut resizer = fast_image_resize::Resizer::new();
    resizer.resize(&source_image, &mut destination_image, &Some(options))
        .map_err(|e| format!("Failed to resize PNG image: {}", e))?;

    Ok(destination_image.buffer().to_vec())
}

fn pixels_data_to_jpeg(pixels: Vec<u8>, width: u32, height: u32, pixel_format: turbojpeg::PixelFormat) -> Result<Vec<u8>, String> {
    let turbo_jpeg_image = turbojpeg::Image {
        pixels,
        width: width as usize,
        pitch: width as usize * pixel_format.size(),
        height: height as usize,
        format: pixel_format,
    };

    let jpeg_buf = compress_jpeg_image(&turbo_jpeg_image)?;
    Ok(jpeg_buf.to_vec())
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
        let expected_len = image_info.width as usize * image_info.height as usize * 
                match image_info.color_type { 
                    png::ColorType::Rgba => 4, 
                    png::ColorType::Rgb  => 3, 
                    _ => return Err("Unsupported color type".into()), 
                };
        if image_bytes.len() != expected_len {
            return Err(format!("Unexpected buffer size: got {}, expected {}", image_bytes.len(), expected_len).into());
        }
        
        match image_info.color_type {
            png::ColorType::Rgba => fast_image_resize::PixelType::U8x4,
            png::ColorType::Rgb => fast_image_resize::PixelType::U8x3,
            _ => return Err("Unsupported color type".into()),
        }
    };

    fn scale(width: u32, height: u32) -> (u32, u32) {
        (width / 8, height / 8)
    }
    let pixels = resize_pixels_data(image_bytes.to_vec(), pixel_type, image_info.width, image_info.height, scale)?;
    let (scaled_width, scaled_height) = scale(image_info.width, image_info.height);

    let turbo_jpeg_pixel_format = match pixel_type {
        fast_image_resize::PixelType::U8x3 => turbojpeg::PixelFormat::RGB,
        fast_image_resize::PixelType::U8x4 => turbojpeg::PixelFormat::RGBA,
        _ => return Err("Unsupported pixel format for JPEG".into()),
    };
    pixels_data_to_jpeg(pixels, scaled_width, scaled_height, turbo_jpeg_pixel_format)
}

// -------------- WEBP --------------

fn make_webp_thumbnail(media_buffer: Vec<u8>) -> Result<Vec<u8>, String> {
    let decoder = webp::Decoder::new(&media_buffer);
    let decoded_webp = decoder.decode()
        .ok_or_else(|| "Failed to decode WebP image (invalid or corrupted data)".to_string())?;

    let width = decoded_webp.width();
    let height = decoded_webp.height();
    let webp_pixel_bytes = decoded_webp.to_vec();


    let pixel_type = {
        let expected_len = width as usize * height as usize * (if decoded_webp.is_alpha() { 4 } else { 3 });
        if webp_pixel_bytes.len() != expected_len {
            return Err(format!("Unexpected buffer size: got {}, expected {}", webp_pixel_bytes.len(), expected_len).into());
        }

        if decoded_webp.is_alpha() {
            fast_image_resize::PixelType::U8x4
        } else {
            fast_image_resize::PixelType::U8x3
        }
    };
    
    fn scale(width: u32, height: u32) -> (u32, u32) {
        (width / 8, height / 8)
    }
    let (scaled_width, scaled_height) = scale(width, height);
    let pixels = resize_pixels_data(webp_pixel_bytes, pixel_type, width, height, scale)?;
    
    let turbo_jpeg_pixel_format = match pixel_type {
        fast_image_resize::PixelType::U8x3 => turbojpeg::PixelFormat::RGB,
        fast_image_resize::PixelType::U8x4 => turbojpeg::PixelFormat::RGBA,
        _ => return Err("Unsupported pixel format for JPEG".into()),
    };
    pixels_data_to_jpeg(pixels, scaled_width, scaled_height, turbo_jpeg_pixel_format)
}
