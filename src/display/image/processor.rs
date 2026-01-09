//! image processing stuff
use {
    crate::display::image::{
        animation::{AnimatedImage, AnimationFrame},
        dimensions::ImageDimensions,
        source::{ImageData, ImageSource},
    },
    color_eyre::eyre::Result,
    image::{DynamicImage, GenericImageView, imageops::FilterType},
};

/// default resampling filter for resizing
const DEFAULT_FILTER: FilterType = FilterType::Lanczos3;

/// image processer
pub struct ImageProcessor {
    /// the dimensions to process with
    target_dimensions: ImageDimensions,

    /// the resampling filter to resize with
    filter: FilterType,
}

impl ImageProcessor {
    /// make a new processor with defaults
    pub fn new() -> Self {
        Self {
            target_dimensions: ImageDimensions::default(),
            filter: DEFAULT_FILTER,
        }
    }

    /// make a new processor with specific target dimensions
    pub fn with_dimensions(dimensions: ImageDimensions) -> Self {
        Self {
            target_dimensions: dimensions,
            filter: DEFAULT_FILTER,
        }
    }

    /// set the resampling filter
    pub fn with_filter(mut self, filter: FilterType) -> Self {
        self.filter = filter;
        self
    }

    /// update the target dimensions
    pub fn set_dimensions(&mut self, dimensions: ImageDimensions) {
        self.target_dimensions = dimensions;
    }

    /// update the resampling filter
    pub fn set_filter(&mut self, filter: FilterType) {
        self.filter = filter;
    }

    /// process an image source into rgba888 data
    pub fn process(&self, source: ImageSource) -> Result<ImageData> {
        let img = source.load()?;
        let original_dimensions = img.dimensions();
        let target_dimensions = self.target_dimensions.compute_target(original_dimensions);

        let resized_img = if original_dimensions != target_dimensions {
            img.resize_exact(target_dimensions.0, target_dimensions.1, self.filter)
        } else {
            img
        };

        Ok(ImageData::from_dynamic_image(resized_img))
    }

    /// Process an image without resizing
    pub fn process_no_resize(&self, source: ImageSource) -> Result<ImageData> {
        let img = source.load()?;
        Ok(ImageData::from_dynamic_image(img))
    }

    /// process an animation, resizing all frames
    pub fn process_animated(&self, mut animated: AnimatedImage) -> Result<AnimatedImage> {
        let orig_dimensions = (animated.width, animated.height);
        let target_dimensions = self.target_dimensions.compute_target(orig_dimensions);

        if orig_dimensions != target_dimensions {
            animated.frames = animated
                .frames
                .into_iter()
                .map(|frame| {
                    let img = DynamicImage::ImageRgba8(
                        image::RgbaImage::from_raw(
                            frame.data.width as u32,
                            frame.data.height as u32,
                            frame.data.rgb_data,
                        )
                        .expect("invalid frame data"),
                    );

                    let resized =
                        img.resize_exact(target_dimensions.0, target_dimensions.1, self.filter);
                    let rgba8 = resized.to_rgba8();
                    let rgba_data = rgba8.into_raw();

                    AnimationFrame {
                        data: ImageData::new(
                            rgba_data,
                            target_dimensions.0 as usize,
                            target_dimensions.1 as usize,
                        ),
                        delay: frame.delay,
                    }
                })
                .collect();

            animated.width = target_dimensions.0;
            animated.height = target_dimensions.1;
        }

        Ok(animated)
    }

    /// extract a single frame from an animation
    pub fn extract_frame(&self, animated: &AnimatedImage, frame_index: usize) -> Result<ImageData> {
        let frame = animated
            .get_frame(frame_index)
            .ok_or_else(|| color_eyre::eyre::eyre!("Frame {} does not exist", frame_index))?;

        let original_dimensions = (frame.data.width as u32, frame.data.height as u32);
        let target_dimensions = self.target_dimensions.compute_target(original_dimensions);

        if original_dimensions != target_dimensions {
            let img = DynamicImage::ImageRgba8(
                image::RgbaImage::from_raw(
                    frame.data.width as u32,
                    frame.data.height as u32,
                    frame.data.rgb_data.clone(),
                )
                .expect("invalid frame data"),
            );

            let resized = img.resize_exact(target_dimensions.0, target_dimensions.1, self.filter);
            Ok(ImageData::from_dynamic_image(resized))
        } else {
            Ok(frame.data.clone())
        }
    }
}

impl Default for ImageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use {super::*, image::DynamicImage};

    #[test]
    fn test_process_no_resize() {
        let img = DynamicImage::new_rgb8(100, 100);
        let source = ImageSource::from_image(img);
        let processor = ImageProcessor::new();

        let result = processor.process_no_resize(source).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 100);
    }

    #[test]
    fn test_process_with_resize() {
        let img = DynamicImage::new_rgb8(200, 200);
        let source = ImageSource::from_image(img);

        let dimensions = ImageDimensions::new(Some(100), None).unwrap();
        let processor = ImageProcessor::with_dimensions(dimensions);

        let result = processor.process(source).unwrap();
        assert_eq!(result.width, 100);
        assert_eq!(result.height, 100);
    }
}
