use anyhow::{Context, Result};
use e6cfg::Cfg;
use icy_sixel::{DiffusionMethod, MethodForLargest, PixelFormat, Quality, sixel_string};
use image::{GenericImageView, ImageReader, imageops::FilterType};
use std::io::Cursor;

pub async fn fetch_remote_file_as_bytes(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .context("Failed to send HTTP request")?;

    if !response.status().is_success() {
        anyhow::bail!("Request failed with status: {}", response.status());
    }

    response
        .bytes()
        .await
        .context("Failed to read response body as bytes")
        .map(|bytes| bytes.to_vec())
}

#[derive(Clone, Copy)]
pub struct ImageDimensions {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl From<&Cfg> for ImageDimensions {
    fn from(cfg: &Cfg) -> Self {
        if let Some(display) = &cfg.display {
            ImageDimensions {
                width: display.width.map(|w| w as u32),
                height: display.height.map(|h| h as u32),
            }
        } else {
            ImageDimensions {
                width: None,
                height: None,
            }
        }
    }
}

fn compute_target_dimensions(original: (u32, u32), target: ImageDimensions) -> (u32, u32) {
    let (orig_width, orig_height) = original;

    match (target.width, target.height) {
        (Some(width), Some(height)) => (width, height),
        (Some(width), None) => {
            let aspect_ratio = orig_height as f32 / orig_width as f32;
            let new_height = (width as f32 * aspect_ratio) as u32;
            (width, new_height)
        }
        (None, Some(height)) => {
            let aspect_ratio = orig_width as f32 / orig_height as f32;
            let new_width = (height as f32 * aspect_ratio) as u32;
            (new_width, height)
        }
        (None, None) => (orig_width, orig_height),
    }
}

pub async fn load_image_from_url(
    url: &str,
    target_dimensions: ImageDimensions,
) -> Result<(Vec<u8>, u32, u32)> {
    let image_data = fetch_remote_file_as_bytes(url)
        .await
        .context("Failed to fetch remote image")?;

    load_image_from_bytes(&image_data, target_dimensions)
}

fn load_image_from_bytes(
    image_data: &[u8],
    target_dimensions: ImageDimensions,
) -> Result<(Vec<u8>, u32, u32)> {
    let cursor = Cursor::new(image_data);
    let mut img = ImageReader::new(cursor)
        .with_guessed_format()
        .context("Failed to guess image format")?
        .decode()
        .context("Failed to decode image")?;

    let original_dimensions = img.dimensions();
    let target_dimensions = compute_target_dimensions(original_dimensions, target_dimensions);

    if original_dimensions != target_dimensions {
        img = img.resize_exact(
            target_dimensions.0,
            target_dimensions.1,
            FilterType::Lanczos3,
        );
    }

    let img_rgb8 = img.to_rgb8();
    let dimensions = img_rgb8.dimensions();
    let img_rgb888 = img_rgb8.into_raw();

    Ok((img_rgb888, dimensions.0, dimensions.1))
}

pub fn convert_rgb888_to_sixel(rgb_data: &[u8], width: u32, height: u32) -> String {
    sixel_string(
        rgb_data,
        width as i32,
        height as i32,
        PixelFormat::RGB888,
        DiffusionMethod::Atkinson,
        MethodForLargest::Auto,
        icy_sixel::MethodForRep::Auto,
        Quality::AUTO,
    )
    .expect("Failed to convert image to sixel string")
}

pub async fn fetch_and_display_image_as_sixel(url: &str) -> Result<()> {
    let cfg = Cfg::get()?;
    let target_dimensions = ImageDimensions::from(&cfg);

    let (img_rgb888, width, height) = load_image_from_url(url, target_dimensions)
        .await
        .context("Failed to load image from URL")?;

    let sixel_data = convert_rgb888_to_sixel(&img_rgb888, width, height);

    println!("{}", sixel_data);
    Ok(())
}

pub async fn fetch_and_display_images_as_sixel(urls: &[&str]) -> Result<()> {
    let cfg = Cfg::get()?;
    let target_dimensions = ImageDimensions::from(&cfg);

    for url in urls {
        let (img_rgb888, width, height) = load_image_from_url(url, target_dimensions)
            .await
            .context("Failed to load image from URL")?;

        let sixel_data = convert_rgb888_to_sixel(&img_rgb888, width, height);

        println!("{}", sixel_data);
    }

    Ok(())
}

pub async fn load_png_as_rgb888_from_url(url: &str) -> Result<(Vec<u8>, u32, u32)> {
    load_image_from_url(
        url,
        ImageDimensions {
            width: None,
            height: None,
        },
    )
    .await
}
