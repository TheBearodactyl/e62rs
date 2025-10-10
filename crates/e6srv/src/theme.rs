#![allow(unused)]

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub base: String,
    pub surface: String,
    pub overlay: String,
    pub muted: String,
    pub subtle: String,
    pub text: String,
    pub love: String,
    pub gold: String,
    pub rose: String,
    pub pine: String,
    pub foam: String,
    pub iris: String,
    pub flamingo: Option<String>,
    pub mauve: Option<String>,
    pub red: Option<String>,
    pub maroon: Option<String>,
    pub peach: Option<String>,
    pub yellow: Option<String>,
    pub green: Option<String>,
    pub teal: Option<String>,
    pub sky: Option<String>,
    pub blue: Option<String>,
    pub lavender: Option<String>,
    pub mantle: Option<String>,
    pub crust: Option<String>,
    pub highlight_low: Option<String>,
    pub highlight_med: Option<String>,
    pub highlight_high: Option<String>,
}

impl ThemeColors {
    #[allow(
        clippy::too_many_arguments,
        reason = "No better way (i say, even though there is most likely a better way)"
    )]
    pub fn new(
        base: String,
        surface: String,
        overlay: String,
        muted: String,
        subtle: String,
        text: String,
        love: String,
        gold: String,
        rose: String,
        pine: String,
        foam: String,
        iris: String,
    ) -> Self {
        Self {
            base,
            surface,
            overlay,
            muted,
            subtle,
            text,
            love,
            gold,
            rose,
            pine,
            foam,
            iris,
            flamingo: None,
            mauve: None,
            red: None,
            maroon: None,
            peach: None,
            yellow: None,
            green: None,
            teal: None,
            sky: None,
            blue: None,
            lavender: None,
            mantle: None,
            crust: None,
            highlight_low: None,
            highlight_med: None,
            highlight_high: None,
        }
    }

    pub fn to_css_vars(&self) -> String {
        let mut vars = format!(
            r#"
            --base: {};
            --surface: {};
            --overlay: {};
            --muted: {};
            --subtle: {};
            --text: {};
            --love: {};
            --gold: {};
            --rose: {};
            --pine: {};
            --foam: {};
            --iris: {};
            "#,
            self.base,
            self.surface,
            self.overlay,
            self.muted,
            self.subtle,
            self.text,
            self.love,
            self.gold,
            self.rose,
            self.pine,
            self.foam,
            self.iris
        );

        if let Some(ref flamingo) = self.flamingo {
            vars.push_str(&format!("--flamingo: {};\n", flamingo));
        }
        if let Some(ref mauve) = self.mauve {
            vars.push_str(&format!("--mauve: {};\n", mauve));
        }
        if let Some(ref red) = self.red {
            vars.push_str(&format!("--red: {};\n", red));
        }
        if let Some(ref maroon) = self.maroon {
            vars.push_str(&format!("--maroon: {};\n", maroon));
        }
        if let Some(ref peach) = self.peach {
            vars.push_str(&format!("--peach: {};\n", peach));
        }
        if let Some(ref yellow) = self.yellow {
            vars.push_str(&format!("--yellow: {};\n", yellow));
        }
        if let Some(ref green) = self.green {
            vars.push_str(&format!("--green: {};\n", green));
        }
        if let Some(ref teal) = self.teal {
            vars.push_str(&format!("--teal: {};\n", teal));
        }
        if let Some(ref sky) = self.sky {
            vars.push_str(&format!("--sky: {};\n", sky));
        }
        if let Some(ref blue) = self.blue {
            vars.push_str(&format!("--blue: {};\n", blue));
        }
        if let Some(ref lavender) = self.lavender {
            vars.push_str(&format!("--lavender: {};\n", lavender));
        }
        if let Some(ref mantle) = self.mantle {
            vars.push_str(&format!("--mantle: {};\n", mantle));
        }
        if let Some(ref crust) = self.crust {
            vars.push_str(&format!("--crust: {};\n", crust));
        }
        if let Some(ref highlight_low) = self.highlight_low {
            vars.push_str(&format!("--highlight-low: {};\n", highlight_low));
        }
        if let Some(ref highlight_med) = self.highlight_med {
            vars.push_str(&format!("--highlight-med: {};\n", highlight_med));
        }
        if let Some(ref highlight_high) = self.highlight_high {
            vars.push_str(&format!("--highlight-high: {};\n", highlight_high));
        }

        vars
    }
}

pub trait Theme {
    fn colors() -> ThemeColors;
    fn name() -> &'static str;
    fn to_css_vars() -> String {
        Self::colors().to_css_vars()
    }
}

#[derive(Clone, Default)]
pub struct RosePine;
#[derive(Clone, Default)]
pub struct RosePineMoon;
#[derive(Clone, Default)]
pub struct RosePineDawn;

impl Theme for RosePine {
    fn colors() -> ThemeColors {
        let mut colors = ThemeColors::new(
            "#191724".to_string(),
            "#1f1d2e".to_string(),
            "#26233a".to_string(),
            "#6e6a86".to_string(),
            "#908caa".to_string(),
            "#e0def4".to_string(),
            "#eb6f92".to_string(),
            "#f6c177".to_string(),
            "#ebbcba".to_string(),
            "#31748f".to_string(),
            "#9ccfd8".to_string(),
            "#c4a7e7".to_string(),
        );

        colors.highlight_low = Some("#21202e".to_string());
        colors.highlight_med = Some("#403d52".to_string());
        colors.highlight_high = Some("#524f67".to_string());

        colors
    }

    fn name() -> &'static str {
        "Rose Pine"
    }
}

impl Theme for RosePineMoon {
    fn colors() -> ThemeColors {
        let mut colors = ThemeColors::new(
            "#232136".to_string(),
            "#2a273f".to_string(),
            "#393552".to_string(),
            "#6e6a86".to_string(),
            "#908caa".to_string(),
            "#e0def4".to_string(),
            "#eb6f92".to_string(),
            "#f6c177".to_string(),
            "#ea9a97".to_string(),
            "#3e8fb0".to_string(),
            "#9ccfd8".to_string(),
            "#c4a7e7".to_string(),
        );

        colors.highlight_low = Some("#2a2d3a".to_string());
        colors.highlight_med = Some("#44415a".to_string());
        colors.highlight_high = Some("#56526e".to_string());

        colors
    }

    fn name() -> &'static str {
        "Rose Pine Moon"
    }
}

impl Theme for RosePineDawn {
    fn colors() -> ThemeColors {
        let mut colors = ThemeColors::new(
            "#faf4ed".to_string(),
            "#fffaf3".to_string(),
            "#f2e9e1".to_string(),
            "#9893a5".to_string(),
            "#797593".to_string(),
            "#575279".to_string(),
            "#b4637a".to_string(),
            "#ea9d34".to_string(),
            "#d7827e".to_string(),
            "#286983".to_string(),
            "#56949f".to_string(),
            "#907aa9".to_string(),
        );

        colors.highlight_low = Some("#f4ede8".to_string());
        colors.highlight_med = Some("#dfdfe0".to_string());
        colors.highlight_high = Some("#cecacd".to_string());

        colors
    }

    fn name() -> &'static str {
        "Rose Pine Dawn"
    }
}

pub struct CatppuccinLatte;
pub struct CatppuccinFrappe;
pub struct CatppuccinMacchiato;
pub struct CatppuccinMocha;

impl Theme for CatppuccinLatte {
    fn colors() -> ThemeColors {
        let mut colors = ThemeColors::new(
            "#eff1f5".to_string(),
            "#e6e9ef".to_string(),
            "#dce0e8".to_string(),
            "#9ca0b0".to_string(),
            "#8c8fa1".to_string(),
            "#4c4f69".to_string(),
            "#dc8a78".to_string(),
            "#fe640b".to_string(),
            "#ea76cb".to_string(),
            "#40a02b".to_string(),
            "#04a5e5".to_string(),
            "#8839ef".to_string(),
        );

        colors.flamingo = Some("#dd7878".to_string());
        colors.mauve = Some("#8839ef".to_string());
        colors.red = Some("#d20f39".to_string());
        colors.maroon = Some("#e64553".to_string());
        colors.peach = Some("#fe640b".to_string());
        colors.yellow = Some("#df8e1d".to_string());
        colors.green = Some("#40a02b".to_string());
        colors.teal = Some("#179299".to_string());
        colors.sky = Some("#04a5e5".to_string());
        colors.blue = Some("#1e66f5".to_string());
        colors.lavender = Some("#7287fd".to_string());
        colors.mantle = Some("#e6e9ef".to_string());
        colors.crust = Some("#dce0e8".to_string());

        colors
    }

    fn name() -> &'static str {
        "Catppuccin Latte"
    }
}

impl Theme for CatppuccinFrappe {
    fn colors() -> ThemeColors {
        let mut colors = ThemeColors::new(
            "#303446".to_string(),
            "#292c3c".to_string(),
            "#232634".to_string(),
            "#737994".to_string(),
            "#838ba7".to_string(),
            "#c6d0f5".to_string(),
            "#f2d5cf".to_string(),
            "#ef9f76".to_string(),
            "#f4b8e4".to_string(),
            "#a6d189".to_string(),
            "#99d1db".to_string(),
            "#ca9ee6".to_string(),
        );

        colors.flamingo = Some("#eebebe".to_string());
        colors.mauve = Some("#ca9ee6".to_string());
        colors.red = Some("#e78284".to_string());
        colors.maroon = Some("#ea999c".to_string());
        colors.peach = Some("#ef9f76".to_string());
        colors.yellow = Some("#e5c890".to_string());
        colors.green = Some("#a6d189".to_string());
        colors.teal = Some("#81c8be".to_string());
        colors.sky = Some("#99d1db".to_string());
        colors.blue = Some("#8caaee".to_string());
        colors.lavender = Some("#babbf1".to_string());
        colors.mantle = Some("#292c3c".to_string());
        colors.crust = Some("#232634".to_string());

        colors
    }

    fn name() -> &'static str {
        "Catppuccin Frappé"
    }
}

impl Theme for CatppuccinMacchiato {
    fn colors() -> ThemeColors {
        let mut colors = ThemeColors::new(
            "#24273a".to_string(),
            "#1e2030".to_string(),
            "#181926".to_string(),
            "#6e738d".to_string(),
            "#8087a2".to_string(),
            "#cad3f5".to_string(),
            "#f4dbd6".to_string(),
            "#f5a97f".to_string(),
            "#f5bde6".to_string(),
            "#a6da95".to_string(),
            "#91d7e3".to_string(),
            "#c6a0f6".to_string(),
        );

        colors.flamingo = Some("#f0c6c6".to_string());
        colors.mauve = Some("#c6a0f6".to_string());
        colors.red = Some("#ed8796".to_string());
        colors.maroon = Some("#ee99a0".to_string());
        colors.peach = Some("#f5a97f".to_string());
        colors.yellow = Some("#eed49f".to_string());
        colors.green = Some("#a6da95".to_string());
        colors.teal = Some("#8bd5ca".to_string());
        colors.sky = Some("#91d7e3".to_string());
        colors.blue = Some("#8aadf4".to_string());
        colors.lavender = Some("#b7bdf8".to_string());
        colors.mantle = Some("#1e2030".to_string());
        colors.crust = Some("#181926".to_string());

        colors
    }

    fn name() -> &'static str {
        "Catppuccin Macchiato"
    }
}

impl Theme for CatppuccinMocha {
    fn colors() -> ThemeColors {
        let mut colors = ThemeColors::new(
            "#1e1e2e".to_string(),
            "#181825".to_string(),
            "#11111b".to_string(),
            "#6c7086".to_string(),
            "#7f849c".to_string(),
            "#cdd6f4".to_string(),
            "#f5e0dc".to_string(),
            "#fab387".to_string(),
            "#f5c2e7".to_string(),
            "#a6e3a1".to_string(),
            "#89dceb".to_string(),
            "#cba6f7".to_string(),
        );

        colors.flamingo = Some("#f2cdcd".to_string());
        colors.mauve = Some("#cba6f7".to_string());
        colors.red = Some("#f38ba8".to_string());
        colors.maroon = Some("#eba0ac".to_string());
        colors.peach = Some("#fab387".to_string());
        colors.yellow = Some("#f9e2af".to_string());
        colors.green = Some("#a6e3a1".to_string());
        colors.teal = Some("#94e2d5".to_string());
        colors.sky = Some("#89dceb".to_string());
        colors.blue = Some("#89b4fa".to_string());
        colors.lavender = Some("#b4befe".to_string());
        colors.mantle = Some("#181825".to_string());
        colors.crust = Some("#11111b".to_string());

        colors
    }

    fn name() -> &'static str {
        "Catppuccin Mocha"
    }
}

pub struct ThemeRegistry {
    themes: HashMap<&'static str, fn() -> ThemeColors>,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            themes: HashMap::new(),
        };

        registry.themes.insert("rose-pine", RosePine::colors);
        registry
            .themes
            .insert("rose-pine-moon", RosePineMoon::colors);
        registry
            .themes
            .insert("rose-pine-dawn", RosePineDawn::colors);

        registry
            .themes
            .insert("catppuccin-latte", CatppuccinLatte::colors);
        registry
            .themes
            .insert("catppuccin-frappe", CatppuccinFrappe::colors);
        registry
            .themes
            .insert("catppuccin-macchiato", CatppuccinMacchiato::colors);
        registry
            .themes
            .insert("catppuccin-mocha", CatppuccinMocha::colors);

        registry
    }

    pub fn get_theme(&self, name: &str) -> Option<ThemeColors> {
        self.themes.get(name).map(|f| f())
    }

    pub fn get_theme_css_vars(&self, name: &str) -> Option<String> {
        self.get_theme(name).map(|colors| colors.to_css_vars())
    }

    pub fn list_themes(&self) -> Vec<&'static str> {
        self.themes.keys().cloned().collect()
    }
}

impl Default for ThemeRegistry {
    fn default() -> Self {
        Self::new()
    }
}
