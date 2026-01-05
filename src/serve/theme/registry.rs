//! theme registry stuff
use {
    crate::serve::theme::{
        Theme, ThemeVariant, metadata::ThemeMetadata, palette::ThemeColors, presets::*,
    },
    hashbrown::HashMap,
};

/// the theme registry
pub struct ThemeRegistry {
    /// the installed themes
    themes: HashMap<&'static str, ThemeMetadata>,
}

impl ThemeRegistry {
    /// make a new registry
    pub fn new() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
        };

        registry.register::<RosePine>("rose-pine");
        registry.register::<RosePineMoon>("rose-pine-moon");
        registry.register::<RosePineDawn>("rose-pine-dawn");
        registry.register::<CatppuccinLatte>("catppuccin-latte");
        registry.register::<CatppuccinFrappe>("catppuccin-frappe");
        registry.register::<CatppuccinMacchiato>("catppuccin-macchiato");
        registry.register::<CatppuccinMocha>("catppuccin-mocha");

        registry
    }

    /// register a theme
    pub fn register<T: Theme>(&mut self, id: &'static str) {
        self.themes.insert(id, ThemeMetadata::new::<T>(id));
    }

    /// get a theme by its id
    pub fn get_theme(&self, id: &str) -> Option<ThemeColors> {
        self.themes.get(id).map(|meta| meta.get_colors())
    }

    /// get the CSS vars of a theme
    pub fn get_theme_css_vars(&self, id: &str) -> Option<String> {
        self.get_theme(id).map(|colors| colors.to_css_vars())
    }

    /// get the metadata of a theme
    pub fn get_metadata(&self, id: &str) -> Option<&ThemeMetadata> {
        self.themes.get(id)
    }

    /// list available themes
    pub fn list_themes(&self) -> Vec<&'static str> {
        self.themes.keys().copied().collect()
    }

    /// list themes by variant
    pub fn list_by_variant(&self, variant: ThemeVariant) -> Vec<&'static str> {
        self.themes
            .iter()
            .filter(|(_, meta)| meta.variant == variant)
            .map(|(id, _)| *id)
            .collect()
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
