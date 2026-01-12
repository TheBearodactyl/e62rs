//! ui themeing stuff
use inquire::ui::{Attributes, Color, RenderConfig, StyleSheet, Styled};

/// get the global render config (rose pine theme)
pub fn get_render_config() -> RenderConfig<'static> {
    let foam = Color::rgb(156, 207, 216);
    let subtle = Color::rgb(144, 140, 170);
    let rose = Color::rgb(235, 188, 186);
    let iris = Color::rgb(196, 167, 231);
    let muted = Color::rgb(110, 106, 134);

    RenderConfig::default()
        .with_prompt_prefix(Styled::new("$").with_fg(rose).with_attr(Attributes::BOLD))
        .with_selected_option(Some(StyleSheet::new().with_fg(foam)))
        .with_selected_checkbox(Styled::new(" [•]").with_fg(iris))
        .with_unselected_checkbox(Styled::new(" [ ]").with_fg(muted))
        .with_help_message(
            StyleSheet::new()
                .with_fg(subtle)
                .with_attr(Attributes::BOLD),
        )
}
