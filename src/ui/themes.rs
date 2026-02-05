//! ui themeing stuff
use inquire::ui::{Attributes, Color, ErrorMessageRenderConfig, RenderConfig, StyleSheet, Styled};

/// get the global render config
pub fn get_render_config() -> RenderConfig<'static> {
    let mauve: Color = Color::rgb(203, 166, 247);
    let red: Color = Color::rgb(243, 139, 168);
    let maroon: Color = Color::rgb(235, 160, 172);
    let green: Color = Color::rgb(166, 227, 161);
    let text: Color = Color::rgb(205, 214, 244);
    let subtext0: Color = Color::rgb(166, 173, 200);
    let overlay1: Color = Color::rgb(127, 132, 156);
    let overlay0: Color = Color::rgb(108, 112, 134);

    RenderConfig {
        prompt_prefix: Styled::new("?").with_fg(mauve),
        answered_prompt_prefix: Styled::new("✔").with_fg(green),
        prompt: StyleSheet::new().with_fg(text).with_attr(Attributes::BOLD),
        default_value: StyleSheet::new().with_fg(overlay0),
        placeholder: StyleSheet::new().with_fg(overlay0),
        help_message: StyleSheet::new().with_fg(subtext0),
        text_input: StyleSheet::new().with_fg(text),
        highlighted_option_prefix: Styled::new("❯").with_fg(mauve),
        option: StyleSheet::new().with_fg(text),
        selected_option: Some(StyleSheet::new().with_fg(green)),
        selected_checkbox: Styled::new("[󰸞]").with_fg(green),
        unselected_checkbox: Styled::new("[ ]").with_fg(overlay0),
        scroll_up_prefix: Styled::new("⇞").with_fg(overlay1),
        scroll_down_prefix: Styled::new("⇟").with_fg(overlay1),
        error_message: ErrorMessageRenderConfig {
            prefix: Styled::new("✘").with_fg(red),
            message: StyleSheet::new().with_fg(red).with_attr(Attributes::BOLD),
            separator: StyleSheet::empty().with_fg(red),
            default_message: "Invalid input",
        },
        canceled_prompt_indicator: Styled::new("✘").with_fg(maroon),
        ..Default::default()
    }
}
