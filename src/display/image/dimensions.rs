//! image dimension handling and validation
use {
    crate::getopt,
    color_eyre::eyre::{Result, bail},
};

/// max allowed dimension to prevent mem exhaustion
const MAX_IMAGE_DIMENSION: u32 = 16000;

/// target dimensions for img processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImageDimensions {
    /// target width (optional)
    width: Option<u32>,

    /// target height (optional)
    height: Option<u32>,
}

impl ImageDimensions {
    /// create new dimensions (validated)
    pub fn new(width: Option<u32>, height: Option<u32>) -> Result<Self> {
        if let Some(w) = width
            && w > MAX_IMAGE_DIMENSION
        {
            bail!("Width {} exceeds maximum {}", w, MAX_IMAGE_DIMENSION);
        }

        if let Some(h) = height
            && h > MAX_IMAGE_DIMENSION
        {
            bail!("Height {} exceeds maximum {}", h, MAX_IMAGE_DIMENSION);
        }

        Ok(Self { width, height })
    }

    /// create dimensions from the loaded config
    pub fn from_cfg(_cfg: &crate::config::options::E62Rs) -> Result<Self> {
        let display = getopt!(display);
        let width = display.width.map(|w| w as u32);
        let height = display.height.map(|h| h as u32);

        Self::new(width, height)
    }

    /// get the target width if specified
    pub fn width(&self) -> Option<u32> {
        self.width
    }

    /// get the target height if specified
    pub fn height(&self) -> Option<u32> {
        self.height
    }

    /// compute target dimensions (preserves aspect ratio)
    pub fn compute_target(&self, original: (u32, u32)) -> (u32, u32) {
        let (orig_width, orig_height) = original;

        match (self.width, self.height) {
            (Some(width), Some(height)) => {
                let width_ratio = width as f64 / orig_width as f64;
                let height_ratio = height as f64 / orig_height as f64;
                let scale = width_ratio.min(height_ratio);
                let new_width = (orig_width as f64 * scale).round() as u32;
                let new_height = (orig_height as f64 * scale).round() as u32;

                (new_width, new_height)
            }

            (Some(width), None) => {
                let aspect_ratio = orig_height as f64 / orig_width as f64;
                let new_height = (width as f64 * aspect_ratio).round() as u32;
                (width, new_height)
            }

            (None, Some(height)) => {
                let aspect_ratio = orig_width as f64 / orig_height as f64;
                let new_width = (height as f64 * aspect_ratio).round() as u32;
                (new_width, height)
            }

            (None, None) => (orig_width, orig_height),
        }
    }
}
