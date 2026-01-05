//! themes for the ui
use {
    demand::Theme,
    std::sync::LazyLock,
    termcolor::{Color, ColorSpec},
};

/// the rose pine colorscheme
trait RosePineTheme {
    /// the rose pine theme
    fn rose_pine() -> Theme;
}

/// make a color
fn make_color(color: Color) -> ColorSpec {
    let mut spec = ColorSpec::new();
    spec.set_fg(Some(color));
    spec
}

impl RosePineTheme for Theme {
    fn rose_pine() -> Self {
        let base = Color::Rgb(25, 23, 36);
        let text = Color::Rgb(224, 222, 244);
        let subtle = Color::Rgb(144, 140, 170);
        let muted = Color::Rgb(110, 106, 134);
        let love = Color::Rgb(235, 111, 146);
        let rose = Color::Rgb(235, 188, 186);
        let foam = Color::Rgb(156, 207, 216);
        let iris = Color::Rgb(196, 167, 231);

        let mut title = make_color(iris);
        title.set_bold(true);

        let mut focused_button = make_color(base);
        focused_button.set_bg(Some(rose));

        let mut blurred_button = make_color(text);
        blurred_button.set_bg(Some(base));

        let mut cursor_style = ColorSpec::new();
        cursor_style
            .set_fg(Some(Color::White))
            .set_bg(Some(Color::Black));

        Self {
            title,
            error_indicator: make_color(love),
            description: make_color(subtle),
            cursor: make_color(rose),
            cursor_str: String::from("❯"),

            selected_prefix: String::from(" [•]"),
            selected_prefix_fg: make_color(foam),
            selected_option: make_color(foam),
            unselected_prefix: String::from(" [ ]"),
            unselected_prefix_fg: make_color(muted),
            unselected_option: make_color(text),

            input_cursor: make_color(rose),
            input_placeholder: make_color(muted),
            input_prompt: make_color(rose),

            help_key: make_color(subtle),
            help_desc: make_color(muted),
            help_sep: make_color(subtle),

            focused_button,
            blurred_button,

            cursor_style,
            force_style: true,
            ..Default::default()
        }
    }
}

/// the rose pine theme
pub static ROSE_PINE: LazyLock<Theme> = LazyLock::new(Theme::rose_pine);
