//! themeing stuff for the gallery
pub mod metadata;
pub mod palette;
pub mod presets;
pub mod registry;

use crate::serve::theme::palette::ThemeColors;

/// a theme
pub trait Theme {
    /// the colors of the theme
    fn colors() -> ThemeColors;
    /// the name of the theme
    fn name() -> &'static str;
    /// the theme variant (light/dark)
    fn variant() -> ThemeVariant;
    /// convert the theme to CSS
    fn to_css_vars() -> String {
        Self::colors().to_css_vars()
    }
}

/// a theme variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    /// light variant
    Light,
    /// dark variant
    Dark,
}
