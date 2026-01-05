//! color parsing
use owo_colors::DynColors;

/// convert string into a `DynColors`
pub fn parse_color(color_str: &str) -> Option<DynColors> {
    use DynColors::Rgb;

    match color_str.to_lowercase().as_str() {
        "red" => Some(Rgb(255, 0, 0)),
        "green" => Some(Rgb(0, 255, 0)),
        "blue" => Some(Rgb(0, 0, 255)),
        "yellow" => Some(Rgb(255, 255, 0)),
        "cyan" => Some(Rgb(0, 255, 255)),
        "magenta" | "purple" => Some(Rgb(255, 0, 255)),
        "black" => Some(Rgb(0, 0, 0)),
        "white" => Some(Rgb(255, 255, 255)),
        "gray" | "grey" => Some(Rgb(128, 128, 128)),
        "orange" => Some(Rgb(255, 165, 0)),
        "pink" => Some(Rgb(255, 192, 203)),
        "brown" => Some(Rgb(139, 69, 19)),
        "artist" => Some(Rgb(240, 176, 0)),
        "character" => Some(Rgb(0, 176, 0)),
        "species" => Some(Rgb(237, 131, 35)),
        "copyright" => Some(Rgb(221, 0, 221)),
        "general" => Some(Rgb(178, 178, 178)),
        "meta" => Some(Rgb(255, 255, 255)),
        "lore" => Some(Rgb(34, 139, 34)),
        "invalid" => Some(Rgb(255, 102, 102)),
        _ => parse_hex_color(color_str),
    }
}

/// parse a hex code into a `DynColors`
fn parse_hex_color(color_str: &str) -> Option<DynColors> {
    let hex = color_str.strip_prefix('#')?;

    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(DynColors::Rgb(r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some(DynColors::Rgb(r * 17, g * 17, b * 17))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_named_colors() {
        assert!(parse_color("red").is_some());
        assert!(parse_color("artist").is_some());
        assert_eq!(parse_color("invalid_color"), None);
    }

    #[test]
    fn test_parse_hex_colors() {
        assert!(parse_color("#FF0000").is_some());
        assert!(parse_color("#F00").is_some());
        assert_eq!(parse_color("#GG0000"), None);
    }
}
