use crate::serve::theme::{Theme, ThemeVariant, palette::ThemeColors};

#[derive(Clone)]
pub struct ThemeMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub variant: ThemeVariant,
    pub colors_fn: fn() -> ThemeColors,
}

impl ThemeMetadata {
    pub fn new<T: Theme>(id: &'static str) -> Self {
        Self {
            id,
            name: T::name(),
            variant: T::variant(),
            colors_fn: T::colors,
        }
    }

    pub fn get_colors(&self) -> ThemeColors {
        (self.colors_fn)()
    }
}
