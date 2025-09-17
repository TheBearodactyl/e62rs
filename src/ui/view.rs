use anyhow::Context;
use anyhow::Result;
use icy_sixel::{sixel_string, DiffusionMethod, MethodForLargest, PixelFormat, Quality};
use image::imageops::FilterType;
use image::GenericImageView;
use image::ImageReader;
use std::io::Cursor;

use crate::config::get_config;

pub async fn fetch_remote_file_as_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .context("Failed to send HTTP request")?;

    if !response.status().is_success() {
        anyhow::bail!("Request failed with status: {}", response.status());
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to read response body as bytes")?;

    Ok(bytes.to_vec())
}

pub async fn load_image_as_rgb888_from_url(url: &str) -> Result<(Vec<u8>, u32, u32)> {
    let cfg = get_config()?;
    let target_width = cfg.display.clone().unwrap().width;
    let target_height = cfg.display.unwrap().height;
    let image_data = fetch_remote_file_as_bytes(url)
        .await
        .context("Failed to fetch remote image")?;

    let cursor = Cursor::new(image_data);
    let mut img = ImageReader::new(cursor)
        .with_guessed_format()
        .context("Failed to guess image format")?
        .decode()
        .context("Failed to decode image")?;

    if let (Some(width), Some(height)) = (target_width, target_height) {
        img = img.resize_exact(width as u32, height as u32, FilterType::Lanczos3);
    } else if let Some(width) = target_width {
        let (orig_width, orig_height) = img.dimensions();
        let aspect_ratio = orig_height as f32 / orig_width as f32;
        let new_height = (width as f32 * aspect_ratio) as u32;
        img = img.resize_exact(width as u32, new_height, FilterType::Lanczos3);
    } else if let Some(height) = target_height {
        let (orig_width, orig_height) = img.dimensions();
        let aspect_ratio = orig_width as f32 / orig_height as f32;
        let new_width = (height as f32 * aspect_ratio) as u32;
        img = img.resize_exact(new_width, height as u32, FilterType::Lanczos3);
    }

    let img_rgb8 = img.to_rgb8();
    let (width, height) = img_rgb8.dimensions();
    let img_rgb888 = img_rgb8.into_raw();

    Ok((img_rgb888, width, height))
}
pub async fn load_png_as_rgb888_from_url(url: &str) -> anyhow::Result<(Vec<u8>, u32, u32)> {
    let image_data = fetch_remote_file_as_bytes(url)
        .await
        .context("Failed to fetch remote image")?;

    let cursor = Cursor::new(image_data);
    let img = ImageReader::new(cursor)
        .decode()
        .context("Failed to load PNG image")?;

    let img_rgb8 = img.to_rgb8();
    let (width, height) = img_rgb8.dimensions();
    let img_rgb888 = img_rgb8.into_raw();

    Ok((img_rgb888, width, height))
}

pub fn convert_rgb888_to_sixel(rgb_data: &[u8], width: u32, height: u32) -> anyhow::Result<String> {
    let sixel_data = sixel_string(
        rgb_data,
        width as i32,
        height as i32,
        PixelFormat::RGB888,
        DiffusionMethod::None,
        MethodForLargest::Auto,
        icy_sixel::MethodForRep::Auto,
        Quality::AUTO,
    )
    .expect("Failed to encode image to sixel format");

    Ok(sixel_data)
}

pub async fn fetch_and_display_image_as_sixel(url: &str) -> anyhow::Result<()> {
    let (img_rgb888, width, height) = load_image_as_rgb888_from_url(url)
        .await
        .context("Failed to load image from URL")?;

    let sixel_data = convert_rgb888_to_sixel(&img_rgb888, width, height)
        .context("Failed to convert image to SIXEL")?;

    print!("{}", sixel_data);
    println!("");

    Ok(())
}
pub async fn fetch_and_display_images_as_sixel(urls: Vec<&str>) -> anyhow::Result<()> {
    for url in urls {
        let (img_rgb888, width, height) = load_image_as_rgb888_from_url(url)
            .await
            .context("Failed to load image from URL")?;

        let sixel_data = convert_rgb888_to_sixel(&img_rgb888, width, height)
            .context("Failed to convert image to SIXEL")?;

        print!("{}", sixel_data);
    }

    println!("");

    Ok(())
}
