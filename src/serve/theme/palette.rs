#[derive(Debug, Clone)]
pub struct CorePalette {
    pub base: String,
    pub surface: String,
    pub overlay: String,
    pub muted: String,
    pub subtle: String,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExtendedPalette {
    pub love: Option<String>,
    pub gold: Option<String>,
    pub rose: Option<String>,
    pub pine: Option<String>,
    pub foam: Option<String>,
    pub iris: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AccentColors {
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
}

#[derive(Debug, Clone, Default)]
pub struct SurfaceColors {
    pub mantle: Option<String>,
    pub crust: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HighlightColors {
    pub low: Option<String>,
    pub med: Option<String>,
    pub high: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub core: CorePalette,
    pub extended: ExtendedPalette,
    pub accents: AccentColors,
    pub surfaces: SurfaceColors,
    pub highlights: HighlightColors,
}

impl ThemeColors {
    pub fn new(core: CorePalette) -> Self {
        Self {
            core,
            extended: ExtendedPalette::default(),
            accents: AccentColors::default(),
            surfaces: SurfaceColors::default(),
            highlights: HighlightColors::default(),
        }
    }

    pub fn with_extended(mut self, extended: ExtendedPalette) -> Self {
        self.extended = extended;
        self
    }

    pub fn with_accents(mut self, accents: AccentColors) -> Self {
        self.accents = accents;
        self
    }

    pub fn with_surfaces(mut self, surfaces: SurfaceColors) -> Self {
        self.surfaces = surfaces;
        self
    }

    pub fn with_highlights(mut self, highlights: HighlightColors) -> Self {
        self.highlights = highlights;
        self
    }

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

    fn add_optional_var(vars: &mut String, name: &str, value: &Option<String>) {
        if let Some(val) = value {
            vars.push_str(&format!("{}: {};\n", name, val));
        }
    }
}
