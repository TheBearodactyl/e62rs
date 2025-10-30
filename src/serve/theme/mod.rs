use crate::serve::theme::palette::ThemeColors;

pub mod metadata;
pub mod palette;
pub mod presets;
pub mod registry;

pub trait Theme {
    fn colors() -> ThemeColors;
    fn name() -> &'static str;
    fn variant() -> ThemeVariant;

    fn to_css_vars() -> String {
        Self::colors().to_css_vars()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeVariant {
    Light,
    Dark,
}
