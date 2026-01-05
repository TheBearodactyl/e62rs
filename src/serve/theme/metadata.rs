//! theme metadata stuff
use crate::serve::theme::{Theme, ThemeVariant, palette::ThemeColors};

/// the metadata of a theme
#[derive(Clone)]
pub struct ThemeMetadata {
    /// the id of the theme
    pub id: &'static str,
    /// the theme name
    pub name: &'static str,
    /// the theme variant (dark/light)
    pub variant: ThemeVariant,
    /// the function returning the themes colors
    pub colors_fn: fn() -> ThemeColors,
}

impl ThemeMetadata {
    /// make new metadata
    pub fn new<T: Theme>(id: &'static str) -> Self {
        Self {
            id,
            name: T::name(),
            variant: T::variant(),
            colors_fn: T::colors,
        }
    }

    /// get the colors
    pub fn get_colors(&self) -> ThemeColors {
        (self.colors_fn)()
    }
}
