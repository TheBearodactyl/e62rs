use owo_colors::OwoColorize;
use std::collections::HashMap;

#[derive(Debug)]
enum ScriptMode {
    Superscript,
    Subscript,
}

fn convert_script(text: &str, mode: ScriptMode) -> String {
    let superscript_map: HashMap<char, char> = [
        ('0', '⁰'),
        ('1', '¹'),
        ('2', '²'),
        ('3', '³'),
        ('4', '⁴'),
        ('5', '⁵'),
        ('6', '⁶'),
        ('7', '⁷'),
        ('8', '⁸'),
        ('9', '⁹'),
        ('+', '⁺'),
        ('-', '⁻'),
        ('=', '⁼'),
        ('(', '⁽'),
        (')', '⁾'),
        ('a', 'ᵃ'),
        ('b', 'ᵇ'),
        ('c', 'ᶜ'),
        ('d', 'ᵈ'),
        ('e', 'ᵉ'),
        ('f', 'ᶠ'),
        ('g', 'ᵍ'),
        ('h', 'ʰ'),
        ('i', 'ⁱ'),
        ('j', 'ʲ'),
        ('k', 'ᵏ'),
        ('l', 'ˡ'),
        ('m', 'ᵐ'),
        ('n', 'ⁿ'),
        ('o', 'ᵒ'),
        ('p', 'ᵖ'),
        ('r', 'ʳ'),
        ('s', 'ˢ'),
        ('t', 'ᵗ'),
        ('u', 'ᵘ'),
        ('v', 'ᵛ'),
        ('w', 'ʷ'),
        ('x', 'ˣ'),
        ('y', 'ʸ'),
        ('z', 'ᶻ'),
        ('A', 'ᴬ'),
        ('B', 'ᴮ'),
        ('D', 'ᴰ'),
        ('E', 'ᴱ'),
        ('G', 'ᴳ'),
        ('H', 'ᴴ'),
        ('I', 'ᴵ'),
        ('J', 'ᴶ'),
        ('K', 'ᴷ'),
        ('L', 'ᴸ'),
        ('M', 'ᴹ'),
        ('N', 'ᴺ'),
        ('O', 'ᴼ'),
        ('P', 'ᴾ'),
        ('R', 'ᴿ'),
        ('T', 'ᵀ'),
        ('U', 'ᵁ'),
        ('V', 'ⱽ'),
        ('W', 'ᵂ'),
    ]
    .iter()
    .cloned()
    .collect();

    let subscript_map: HashMap<char, char> = [
        ('0', '₀'),
        ('1', '₁'),
        ('2', '₂'),
        ('3', '₃'),
        ('4', '₄'),
        ('5', '₅'),
        ('6', '₆'),
        ('7', '₇'),
        ('8', '₈'),
        ('9', '₉'),
        ('+', '₊'),
        ('-', '₋'),
        ('=', '₌'),
        ('(', '₍'),
        (')', '₎'),
        ('a', 'ₐ'),
        ('e', 'ₑ'),
        ('h', 'ₕ'),
        ('i', 'ᵢ'),
        ('j', 'ⱼ'),
        ('k', 'ₖ'),
        ('l', 'ₗ'),
        ('m', 'ₘ'),
        ('n', 'ₙ'),
        ('o', 'ₒ'),
        ('p', 'ₚ'),
        ('r', 'ᵣ'),
        ('s', 'ₛ'),
        ('t', 'ₜ'),
        ('u', 'ᵤ'),
        ('v', 'ᵥ'),
        ('x', 'ₓ'),
    ]
    .iter()
    .cloned()
    .collect();

    let map = match mode {
        ScriptMode::Superscript => &superscript_map,
        ScriptMode::Subscript => &subscript_map,
    };

    text.chars()
        .map(|c| map.get(&c).cloned().unwrap_or(c))
        .collect()
}

pub fn format_text(input: &str) -> String {
    fn parse_color(color_str: &str) -> Option<owo_colors::DynColors> {
        use owo_colors::DynColors::*;

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

            _ => {
                if let Some(hex) = color_str.strip_prefix('#') {
                    if hex.len() == 6 {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            u8::from_str_radix(&hex[0..2], 16),
                            u8::from_str_radix(&hex[2..4], 16),
                            u8::from_str_radix(&hex[4..6], 16),
                        ) {
                            return Some(Rgb(r, g, b));
                        }
                    } else if hex.len() == 3 {
                        if let (Ok(r), Ok(g), Ok(b)) = (
                            u8::from_str_radix(&hex[0..1], 16),
                            u8::from_str_radix(&hex[1..2], 16),
                            u8::from_str_radix(&hex[2..3], 16),
                        ) {
                            return Some(Rgb(r * 17, g * 17, b * 17));
                        }
                    }
                }
                None
            }
        }
    }

    fn parse(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut result = String::new();

        while let Some(&c) = chars.peek() {
            match c {
                '[' => {
                    chars.next();
                    let mut tag = String::new();
                    let mut tag_param = None;

                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch == ']' {
                            break;
                        } else if ch == '=' && tag_param.is_none() {
                            tag_param = Some(String::new());
                        } else if let Some(ref mut param) = tag_param {
                            param.push(ch);
                        } else {
                            tag.push(ch);
                        }
                    }

                    let tag_lower = tag.to_lowercase();
                    let closing_tag = if tag_param.is_some() {
                        format!("[/{}]", tag_lower)
                    } else {
                        format!("[/{}]", tag)
                    };

                    let inner = parse_until(chars, &closing_tag);

                    let formatted_inner = match tag_lower.as_str() {
                        "b" => inner.bold().to_string(),
                        "i" => inner.italic().to_string(),
                        "u" => inner.underline().to_string(),
                        "s" => inner.strikethrough().to_string(),
                        "sup" => convert_script(&inner, ScriptMode::Superscript),
                        "sub" => convert_script(&inner, ScriptMode::Subscript),
                        "spoiler" => format!(
                            "{}{}{}",
                            "░".black().on_black(),
                            inner.black().on_black(),
                            "░".black().on_black()
                        ),
                        "color" => {
                            if let Some(ref color_str) = tag_param {
                                if let Some(color) = parse_color(color_str) {
                                    inner.color(color).to_string()
                                } else {
                                    inner
                                }
                            } else {
                                inner
                            }
                        }
                        "quote" => inner
                            .lines()
                            .map(|line| format!("{} {}", "│".bright_black(), line))
                            .collect::<Vec<_>>()
                            .join("\n"),
                        "code" => {
                            format!("{}", inner.on_bright_black())
                        }
                        "section" => {
                            let title = tag_param.as_deref().unwrap_or("Section");
                            format!("┌─ {}\n{}\n└─", title.bright_cyan(), inner)
                        }
                        _ => inner,
                    };

                    result.push_str(&formatted_inner);
                }

                '`' => {
                    chars.next();
                    let mut code = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '`' {
                            chars.next();
                            break;
                        }
                        code.push(ch);
                        chars.next();
                    }
                    result.push_str(&code.on_bright_black().to_string());
                }

                '"' => {
                    chars.next();
                    let mut text = String::new();
                    let mut escaped = false;

                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if escaped {
                            text.push(ch);
                            escaped = false;
                        } else if ch == '\\' {
                            escaped = true;
                        } else if ch == '"' {
                            break;
                        } else {
                            text.push(ch);
                        }
                    }

                    if chars.peek() == Some(&':') {
                        chars.next();
                        let mut url = String::new();

                        if chars.peek() == Some(&'[') {
                            chars.next();
                            while let Some(&ch) = chars.peek() {
                                if ch == ']' {
                                    chars.next();
                                    break;
                                }
                                url.push(ch);
                                chars.next();
                            }
                        } else {
                            while let Some(&ch) = chars.peek() {
                                if ch.is_whitespace() {
                                    break;
                                }
                                url.push(ch);
                                chars.next();
                            }
                        }

                        result.push_str(&format!(
                            "\x1b]8;;{}\x07{}\x1b]8;;\x07",
                            url,
                            text.bright_blue().underline()
                        ));
                    } else {
                        result.push('"');
                        result.push_str(&text);
                        result.push('"');
                    }
                }

                '{' if chars.clone().nth(1) == Some('{') => {
                    chars.next();
                    chars.next();
                    let mut search = String::new();
                    let mut depth = 0;

                    while let Some(&ch) = chars.peek() {
                        if ch == '{' && chars.clone().nth(1) == Some('{') {
                            depth += 1;
                            search.push_str("{{");
                            chars.next();
                            chars.next();
                        } else if ch == '}' && chars.clone().nth(1) == Some('}') {
                            if depth == 0 {
                                chars.next();
                                chars.next();
                                break;
                            } else {
                                depth -= 1;
                                search.push_str("}}");
                                chars.next();
                                chars.next();
                            }
                        } else {
                            search.push(ch);
                            chars.next();
                        }
                    }

                    result.push_str(&format!("[{}]", search.bright_blue().underline()));
                }

                '[' if chars.clone().nth(1) == Some('[') => {
                    chars.next();
                    chars.next();
                    let mut wiki_link = String::new();
                    let mut display_text = None;

                    while let Some(&ch) = chars.peek() {
                        if ch == '|' && display_text.is_none() {
                            display_text = Some(String::new());
                            chars.next();
                        } else if ch == ']' && chars.clone().nth(1) == Some(']') {
                            chars.next();
                            chars.next();
                            break;
                        } else if let Some(ref mut text) = display_text {
                            text.push(ch);
                            chars.next();
                        } else {
                            wiki_link.push(ch);
                            chars.next();
                        }
                    }

                    let link_text = display_text.as_deref().unwrap_or(&wiki_link);
                    result.push_str(&link_text.bright_blue().underline().to_string());
                }

                'h' if chars.clone().take(3).collect::<String>().starts_with("h")
                    && chars.clone().nth(1).is_some_and(|c| c.is_ascii_digit())
                    && chars.clone().nth(2) == Some('.') =>
                {
                    chars.next();
                    let level = chars.next().unwrap();
                    chars.next();

                    while chars.peek() == Some(&' ') {
                        chars.next();
                    }

                    let mut header_text = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '\n' {
                            break;
                        }
                        header_text.push(ch);
                        chars.next();
                    }

                    let formatted = match level {
                        '1' => header_text.bold().bright_white().to_string(),
                        '2' => header_text.bold().white().to_string(),
                        '3' => header_text.bold().to_string(),
                        '4' => header_text.underline().to_string(),
                        '5' => header_text.italic().to_string(),
                        _ => header_text,
                    };

                    result.push_str(&formatted);
                }

                '*' if chars.clone().nth(1) == Some(' ') || chars.clone().nth(1) == Some('*') => {
                    let mut level: usize = 0;
                    while chars.peek() == Some(&'*') {
                        level += 1;
                        chars.next();
                    }

                    if chars.peek() == Some(&' ') {
                        chars.next();
                    }

                    let indent = "  ".repeat(level.saturating_sub(1));
                    let bullet = if level == 1 { "•" } else { "◦" };
                    result.push_str(&format!("{}{} ", indent, bullet.bright_black()));
                }

                _ => {
                    result.push(c);
                    chars.next();
                }
            }
        }

        result
    }

    fn parse_until(chars: &mut std::iter::Peekable<std::str::Chars>, closing_tag: &str) -> String {
        let mut buffer = String::new();

        while chars.peek().is_some() {
            let mut lookahead = chars.clone();
            let mut possible_tag = String::new();

            for _ in 0..closing_tag.len() {
                if let Some(ch) = lookahead.next() {
                    possible_tag.push(ch);
                } else {
                    break;
                }
            }

            if possible_tag.to_lowercase() == closing_tag.to_lowercase() {
                for _ in 0..closing_tag.len() {
                    chars.next();
                }
                break;
            } else {
                buffer.push(chars.next().unwrap());
            }
        }

        let mut inner_chars = buffer.chars().peekable();
        parse(&mut inner_chars)
    }

    let mut chars = input.chars().peekable();
    parse(&mut chars)
}
