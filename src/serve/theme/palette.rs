//! pallete creation stuff

/// the core colors every theme needs
#[derive(Debug, Clone)]
pub struct CorePalette {
    /// main background color
    pub base: String,
    /// slightly elevated surface color
    pub surface: String,
    /// overlay/popup background color
    pub overlay: String,
    /// muted foreground color for less important text
    pub muted: String,
    /// subtle foreground color
    pub subtle: String,
    /// main text color
    pub text: String,
}

/// optional extended palette colors (rose pine style)
#[derive(Debug, Clone, Default)]
pub struct ExtendedPalette {
    /// love/red accent
    pub love: Option<String>,
    /// gold/yellow accent
    pub gold: Option<String>,
    /// rose/pink accent
    pub rose: Option<String>,
    /// pine/green accent
    pub pine: Option<String>,
    /// foam/cyan accent
    pub foam: Option<String>,
    /// iris/purple accent
    pub iris: Option<String>,
}

/// optional accent colors (catppuccin style)
#[derive(Debug, Clone, Default)]
pub struct AccentColors {
    /// flamingo accent
    pub flamingo: Option<String>,
    /// mauve accent
    pub mauve: Option<String>,
    /// red accent
    pub red: Option<String>,
    /// maroon accent
    pub maroon: Option<String>,
    /// peach accent
    pub peach: Option<String>,
    /// yellow accent
    pub yellow: Option<String>,
    /// green accent
    pub green: Option<String>,
    /// teal accent
    pub teal: Option<String>,
    /// sky accent
    pub sky: Option<String>,
    /// blue accent
    pub blue: Option<String>,
    /// lavender accent
    pub lavender: Option<String>,
}

/// optional surface color variants
#[derive(Debug, Clone, Default)]
pub struct SurfaceColors {
    /// mantle layer (below base)
    pub mantle: Option<String>,
    /// crust layer (lowest)
    pub crust: Option<String>,
}

/// optional highlight colors for selections etc
#[derive(Debug, Clone, Default)]
pub struct HighlightColors {
    /// low intensity highlight
    pub low: Option<String>,
    /// medium intensity highlight
    pub med: Option<String>,
    /// high intensity highlight
    pub high: Option<String>,
}

/// complete theme color configuration
#[derive(Debug, Clone)]
pub struct ThemeColors {
    /// required core palette
    pub core: CorePalette,
    /// optional extended palette
    pub extended: ExtendedPalette,
    /// optional accent colors
    pub accents: AccentColors,
    /// optional surface colors
    pub surfaces: SurfaceColors,
    /// optional highlight colors
    pub highlights: HighlightColors,
}

impl ThemeColors {
    /// make new theme colors
    pub fn new(core: CorePalette) -> Self {
        Self {
            core,
            extended: ExtendedPalette::default(),
            accents: AccentColors::default(),
            surfaces: SurfaceColors::default(),
            highlights: HighlightColors::default(),
        }
    }

    /// set the extension pallete
    pub fn with_extended(mut self, extended: ExtendedPalette) -> Self {
        self.extended = extended;
        self
    }

    /// set the accent colors
    pub fn with_accents(mut self, accents: AccentColors) -> Self {
        self.accents = accents;
        self
    }

    /// set the surface colors
    pub fn with_surfaces(mut self, surfaces: SurfaceColors) -> Self {
        self.surfaces = surfaces;
        self
    }

    /// set the highlight colors
    pub fn with_highlights(mut self, highlights: HighlightColors) -> Self {
        self.highlights = highlights;
        self
    }

    /// convert the pallete to CSS
    pub fn to_css_vars(&self) -> String {
        let mut vars = String::new();

        vars.push_str(&format!(
            "--base: {};\n--surface: {};\n--overlay: {};\n--muted: {};\n--subtle: {};\n--text: {};\n",
            self.core.base, self.core.surface, self.core.overlay,
            self.core.muted, self.core.subtle, self.core.text
        ));

        Self::add_optional_var(&mut vars, "--love", &self.extended.love);
        Self::add_optional_var(&mut vars, "--gold", &self.extended.gold);
        Self::add_optional_var(&mut vars, "--rose", &self.extended.rose);
        Self::add_optional_var(&mut vars, "--pine", &self.extended.pine);
        Self::add_optional_var(&mut vars, "--foam", &self.extended.foam);
        Self::add_optional_var(&mut vars, "--iris", &self.extended.iris);
        Self::add_optional_var(&mut vars, "--flamingo", &self.accents.flamingo);
        Self::add_optional_var(&mut vars, "--mauve", &self.accents.mauve);
        Self::add_optional_var(&mut vars, "--red", &self.accents.red);
        Self::add_optional_var(&mut vars, "--maroon", &self.accents.maroon);
        Self::add_optional_var(&mut vars, "--peach", &self.accents.peach);
        Self::add_optional_var(&mut vars, "--yellow", &self.accents.yellow);
        Self::add_optional_var(&mut vars, "--green", &self.accents.green);
        Self::add_optional_var(&mut vars, "--teal", &self.accents.teal);
        Self::add_optional_var(&mut vars, "--sky", &self.accents.sky);
        Self::add_optional_var(&mut vars, "--blue", &self.accents.blue);
        Self::add_optional_var(&mut vars, "--lavender", &self.accents.lavender);
        Self::add_optional_var(&mut vars, "--mantle", &self.surfaces.mantle);
        Self::add_optional_var(&mut vars, "--crust", &self.surfaces.crust);
        Self::add_optional_var(&mut vars, "--highlight-low", &self.highlights.low);
        Self::add_optional_var(&mut vars, "--highlight-med", &self.highlights.med);
        Self::add_optional_var(&mut vars, "--highlight-high", &self.highlights.high);

        vars
    }

    /// helper to add an optional css variable
    fn add_optional_var(vars: &mut String, name: &str, value: &Option<String>) {
        if let Some(val) = value {
            vars.push_str(&format!("{}: {};\n", name, val));
        }
    }
}
