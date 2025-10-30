use {
    crate::serve::theme::{
        Theme, ThemeVariant, metadata::ThemeMetadata, palette::ThemeColors, presets::*,
    },
    std::collections::HashMap,
};

pub struct ThemeRegistry {
    themes: HashMap<&'static str, ThemeMetadata>,
}

impl ThemeRegistry {
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

    pub fn register<T: Theme>(&mut self, id: &'static str) {
        self.themes.insert(id, ThemeMetadata::new::<T>(id));
    }

    pub fn get_theme(&self, id: &str) -> Option<ThemeColors> {
        self.themes.get(id).map(|meta| meta.get_colors())
    }

    pub fn get_theme_css_vars(&self, id: &str) -> Option<String> {
        self.get_theme(id).map(|colors| colors.to_css_vars())
    }

    pub fn get_metadata(&self, id: &str) -> Option<&ThemeMetadata> {
        self.themes.get(id)
    }

    pub fn list_themes(&self) -> Vec<&'static str> {
        self.themes.keys().copied().collect()
    }

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
