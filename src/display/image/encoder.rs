//! image encoding stuff
use {
    crate::display::image::source::ImageData,
    color_eyre::eyre::{Context, Result},
    icy_sixel::EncodeOptions,
};

/// encodes rgb image data to sixel format
pub struct SixelEncoder {
    /// encoder options
    options: EncodeOptions,
}

impl SixelEncoder {
    /// makes a new encoder using defaults
    pub fn new() -> Self {
        Self {
            options: EncodeOptions::default(),
        }
    }

    /// makes a new encoder with custom encoding options
    pub fn with_options(options: EncodeOptions) -> Self {
        Self { options }
    }

    /// update the encoding options
    pub fn set_options(&mut self, options: EncodeOptions) {
        self.options = options;
    }

    /// encode image data to a sixel string
    pub fn encode(&self, data: &ImageData) -> Result<String> {
        icy_sixel::sixel_encode(&data.rgb_data, data.width, data.height, &self.options)
            .context("Failed to encode sixel")
            .map(|s| s.to_string())
    }

    /// encode raw rgb888 data to sixel format
    pub fn encode_raw(&self, rgb_data: &[u8], width: usize, height: usize) -> Result<String> {
        icy_sixel::sixel_encode(rgb_data, width, height, &self.options)
            .context("Failed to encode sixel")
            .map(|s| s.to_string())
    }
}

impl Default for SixelEncoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_small_image() {
        let encoder = SixelEncoder::new();
        let rgb_data = vec![255u8; 10 * 10 * 3];
        let image_data = ImageData::new(rgb_data, 10, 10);

        let result = encoder.encode(&image_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encode_raw() {
        let encoder = SixelEncoder::new();
        let rgb_data = vec![0u8; 5 * 5 * 3];

        let result = encoder.encode_raw(&rgb_data, 5, 5);
        assert!(result.is_ok());
    }
}
