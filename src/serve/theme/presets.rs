use crate::serve::theme::{Theme, ThemeVariant, palette::*};

macro_rules! impl_theme {
    ($name:ident, $display_name:expr, $variant:expr, $colors:expr) => {
        #[derive(Clone, Default)]
        pub struct $name;

        impl Theme for $name {
            fn colors() -> ThemeColors {
                $colors
            }

            fn name() -> &'static str {
                $display_name
            }

            fn variant() -> ThemeVariant {
                $variant
            }
        }
    };
}

impl_theme!(RosePine, "Rose Pine", ThemeVariant::Dark, {
    ThemeColors::new(CorePalette {
        base: "#191724".to_string(),
        surface: "#1f1d2e".to_string(),
        overlay: "#26233a".to_string(),
        muted: "#6e6a86".to_string(),
        subtle: "#908caa".to_string(),
        text: "#e0def4".to_string(),
    })
    .with_extended(ExtendedPalette {
        love: Some("#eb6f92".to_string()),
        gold: Some("#f6c177".to_string()),
        rose: Some("#ebbcba".to_string()),
        pine: Some("#31748f".to_string()),
        foam: Some("#9ccfd8".to_string()),
        iris: Some("#c4a7e7".to_string()),
    })
    .with_highlights(HighlightColors {
        low: Some("#21202e".to_string()),
        med: Some("#403d52".to_string()),
        high: Some("#524f67".to_string()),
    })
});

impl_theme!(RosePineMoon, "Rose Pine Moon", ThemeVariant::Dark, {
    ThemeColors::new(CorePalette {
        base: "#232136".to_string(),
        surface: "#2a273f".to_string(),
        overlay: "#393552".to_string(),
        muted: "#6e6a86".to_string(),
        subtle: "#908caa".to_string(),
        text: "#e0def4".to_string(),
    })
    .with_extended(ExtendedPalette {
        love: Some("#eb6f92".to_string()),
        gold: Some("#f6c177".to_string()),
        rose: Some("#ea9a97".to_string()),
        pine: Some("#3e8fb0".to_string()),
        foam: Some("#9ccfd8".to_string()),
        iris: Some("#c4a7e7".to_string()),
    })
    .with_highlights(HighlightColors {
        low: Some("#2a2d3a".to_string()),
        med: Some("#44415a".to_string()),
        high: Some("#56526e".to_string()),
    })
});

impl_theme!(RosePineDawn, "Rose Pine Dawn", ThemeVariant::Light, {
    ThemeColors::new(CorePalette {
        base: "#faf4ed".to_string(),
        surface: "#fffaf3".to_string(),
        overlay: "#f2e9e1".to_string(),
        muted: "#9893a5".to_string(),
        subtle: "#797593".to_string(),
        text: "#575279".to_string(),
    })
    .with_extended(ExtendedPalette {
        love: Some("#b4637a".to_string()),
        gold: Some("#ea9d34".to_string()),
        rose: Some("#d7827e".to_string()),
        pine: Some("#286983".to_string()),
        foam: Some("#56949f".to_string()),
        iris: Some("#907aa9".to_string()),
    })
    .with_highlights(HighlightColors {
        low: Some("#f4ede8".to_string()),
        med: Some("#dfdfe0".to_string()),
        high: Some("#cecacd".to_string()),
    })
});

impl_theme!(CatppuccinLatte, "Catppuccin Latte", ThemeVariant::Light, {
    ThemeColors::new(CorePalette {
        base: "#eff1f5".to_string(),
        surface: "#e6e9ef".to_string(),
        overlay: "#dce0e8".to_string(),
        muted: "#9ca0b0".to_string(),
        subtle: "#8c8fa1".to_string(),
        text: "#4c4f69".to_string(),
    })
    .with_extended(ExtendedPalette {
        love: Some("#dc8a78".to_string()),
        gold: Some("#fe640b".to_string()),
        rose: Some("#ea76cb".to_string()),
        pine: Some("#40a02b".to_string()),
        foam: Some("#04a5e5".to_string()),
        iris: Some("#8839ef".to_string()),
    })
    .with_accents(AccentColors {
        flamingo: Some("#dd7878".to_string()),
        mauve: Some("#8839ef".to_string()),
        red: Some("#d20f39".to_string()),
        maroon: Some("#e64553".to_string()),
        peach: Some("#fe640b".to_string()),
        yellow: Some("#df8e1d".to_string()),
        green: Some("#40a02b".to_string()),
        teal: Some("#179299".to_string()),
        sky: Some("#04a5e5".to_string()),
        blue: Some("#1e66f5".to_string()),
        lavender: Some("#7287fd".to_string()),
    })
    .with_surfaces(SurfaceColors {
        mantle: Some("#e6e9ef".to_string()),
        crust: Some("#dce0e8".to_string()),
    })
});

impl_theme!(CatppuccinFrappe, "Catppuccin Frappé", ThemeVariant::Dark, {
    ThemeColors::new(CorePalette {
        base: "#303446".to_string(),
        surface: "#292c3c".to_string(),
        overlay: "#232634".to_string(),
        muted: "#737994".to_string(),
        subtle: "#838ba7".to_string(),
        text: "#c6d0f5".to_string(),
    })
    .with_extended(ExtendedPalette {
        love: Some("#f2d5cf".to_string()),
        gold: Some("#ef9f76".to_string()),
        rose: Some("#f4b8e4".to_string()),
        pine: Some("#a6d189".to_string()),
        foam: Some("#99d1db".to_string()),
        iris: Some("#ca9ee6".to_string()),
    })
    .with_accents(AccentColors {
        flamingo: Some("#eebebe".to_string()),
        mauve: Some("#ca9ee6".to_string()),
        red: Some("#e78284".to_string()),
        maroon: Some("#ea999c".to_string()),
        peach: Some("#ef9f76".to_string()),
        yellow: Some("#e5c890".to_string()),
        green: Some("#a6d189".to_string()),
        teal: Some("#81c8be".to_string()),
        sky: Some("#99d1db".to_string()),
        blue: Some("#8caaee".to_string()),
        lavender: Some("#babbf1".to_string()),
    })
    .with_surfaces(SurfaceColors {
        mantle: Some("#292c3c".to_string()),
        crust: Some("#232634".to_string()),
    })
});

impl_theme!(
    CatppuccinMacchiato,
    "Catppuccin Macchiato",
    ThemeVariant::Dark,
    {
        ThemeColors::new(CorePalette {
            base: "#24273a".to_string(),
            surface: "#1e2030".to_string(),
            overlay: "#181926".to_string(),
            muted: "#6e738d".to_string(),
            subtle: "#8087a2".to_string(),
            text: "#cad3f5".to_string(),
        })
        .with_extended(ExtendedPalette {
            love: Some("#f4dbd6".to_string()),
            gold: Some("#f5a97f".to_string()),
            rose: Some("#f5bde6".to_string()),
            pine: Some("#a6da95".to_string()),
            foam: Some("#91d7e3".to_string()),
            iris: Some("#c6a0f6".to_string()),
        })
        .with_accents(AccentColors {
            flamingo: Some("#f0c6c6".to_string()),
            mauve: Some("#c6a0f6".to_string()),
            red: Some("#ed8796".to_string()),
            maroon: Some("#ee99a0".to_string()),
            peach: Some("#f5a97f".to_string()),
            yellow: Some("#eed49f".to_string()),
            green: Some("#a6da95".to_string()),
            teal: Some("#8bd5ca".to_string()),
            sky: Some("#91d7e3".to_string()),
            blue: Some("#8aadf4".to_string()),
            lavender: Some("#b7bdf8".to_string()),
        })
        .with_surfaces(SurfaceColors {
            mantle: Some("#1e2030".to_string()),
            crust: Some("#181926".to_string()),
        })
    }
);

impl_theme!(CatppuccinMocha, "Catppuccin Mocha", ThemeVariant::Dark, {
    ThemeColors::new(CorePalette {
        base: "#1e1e2e".to_string(),
        surface: "#181825".to_string(),
        overlay: "#11111b".to_string(),
        muted: "#6c7086".to_string(),
        subtle: "#7f849c".to_string(),
        text: "#cdd6f4".to_string(),
    })
    .with_extended(ExtendedPalette {
        love: Some("#f5e0dc".to_string()),
        gold: Some("#fab387".to_string()),
        rose: Some("#f5c2e7".to_string()),
        pine: Some("#a6e3a1".to_string()),
        foam: Some("#89dceb".to_string()),
        iris: Some("#cba6f7".to_string()),
    })
    .with_accents(AccentColors {
        flamingo: Some("#f2cdcd".to_string()),
        mauve: Some("#cba6f7".to_string()),
        red: Some("#f38ba8".to_string()),
        maroon: Some("#eba0ac".to_string()),
        peach: Some("#fab387".to_string()),
        yellow: Some("#f9e2af".to_string()),
        green: Some("#a6e3a1".to_string()),
        teal: Some("#94e2d5".to_string()),
        sky: Some("#89dceb".to_string()),
        blue: Some("#89b4fa".to_string()),
        lavender: Some("#b4befe".to_string()),
    })
    .with_surfaces(SurfaceColors {
        mantle: Some("#181825".to_string()),
        crust: Some("#11111b".to_string()),
    })
});
