//! image processing stuff
use {
    crate::display::image::{
        dimensions::ImageDimensions,
        source::{ImageData, ImageSource},
    },
    color_eyre::eyre::Result,
    image::{GenericImageView, imageops::FilterType},
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

    /// process an image source into rgb888 data
    pub fn process(&self, source: ImageSource) -> Result<ImageData> {
        let img = source.load()?;
        let original_dimensions = img.dimensions();
        let target_dimensions = self.target_dimensions.compute_target(original_dimensions);

        let resized_img = if original_dimensions != target_dimensions {
            img.resize(target_dimensions.0, target_dimensions.1, self.filter)
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
