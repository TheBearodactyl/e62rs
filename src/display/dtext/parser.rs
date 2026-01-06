//! dtext parser
use {
    crate::{
        display::dtext::{
            color::parse_color,
            script::{ScriptMode, convert_script},
        },
        mkstr,
        utils::IteratorRepeatExt,
    },
    owo_colors::OwoColorize,
    std::{iter::Peekable, str::Chars},
};

/// format a dtext string
pub fn format_text(input: &str) -> String {
    let mut chars = input.chars().peekable();
    parse(&mut chars)
}

/// parse a character stream into formatted output
pub fn parse(chars: &mut Peekable<Chars>) -> String {
    mkstr!(result, chars.size_hint().0);

    while let Some(&c) = chars.peek() {
        match c {
            '[' if matches_ahead(chars, "[[") => {
                handle_wiki_link(chars, &mut result);
            }
            '[' => {
                handle_tag(chars, &mut result);
            }
            '`' => {
                handle_inline_code(chars, &mut result);
            }
            '"' => {
                handle_link(chars, &mut result);
            }
            '{' if matches_ahead(chars, "{{") => {
                handle_search(chars, &mut result);
            }
            'h' if is_header_start(chars) => {
                handle_header(chars, &mut result);
            }
            '*' if is_list_start(chars) => {
                handle_list(chars, &mut result);
            }
            _ => {
                result.push(c);
                chars.next();
            }
        }
    }

    result
}

/// checks if the next characters match a pattern
pub fn matches_ahead(chars: &Peekable<Chars>, pattern: &str) -> bool {
    let mut lookahead = chars.clone();
    pattern
        .chars()
        .all(|expected| lookahead.next() == Some(expected))
}

/// check if the current pos is the start of a header
pub fn is_header_start(chars: &Peekable<Chars>) -> bool {
    let mut lookahead = chars.clone();
    matches!(lookahead.next(), Some('h'))
        && matches!(lookahead.next(), Some(c) if c.is_ascii_digit())
        && matches!(lookahead.next(), Some('.'))
}

/// check if the current pos is the start of a list
pub fn is_list_start(chars: &Peekable<Chars>) -> bool {
    chars.clone().nth(1).is_some_and(|c| c == ' ' || c == '*')
}

/// handle wiki-style links
pub fn handle_wiki_link(chars: &mut Peekable<Chars>, result: &mut String) {
    chars.skip_n(2);

    mkstr!(link_text);
    let mut display_text: Option<String> = None;

    while let Some(&ch) = chars.peek() {
        match ch {
            '|' if display_text.is_none() => {
                display_text = Some(String::new());
                chars.next();
            }
            ']' if matches_ahead(chars, "]]") => {
                chars.next();
                chars.next();
                break;
            }
            _ => {
                chars.next();
                if let Some(ref mut dt) = display_text {
                    dt.push(ch);
                } else {
                    link_text.push(ch);
                }
            }
        }
    }

    let display = display_text.as_deref().unwrap_or(&link_text);
    result.push_str(&display.bright_blue().underline().to_string());
}

/// handles dtext tags
pub fn handle_tag(chars: &mut Peekable<Chars>, result: &mut String) {
    chars.next();

    mkstr!(tag);
    let mut param: Option<String> = None;

    while let Some(&ch) = chars.peek() {
        chars.next();
        match ch {
            ']' => break,
            '=' if param.is_none() => {
                param = Some(String::new());
            }
            _ => {
                if let Some(ref mut p) = param {
                    p.push(ch);
                } else {
                    tag.push(ch);
                }
            }
        }
    }

    let tag_lower = tag.to_lowercase();
    let closing_tag = format!("[/{}]", if param.is_some() { &tag_lower } else { &tag });

    let inner = parse_until(chars, &closing_tag);
    let formatted = apply_tag_formatting(&tag_lower, param.as_deref(), &inner);

    result.push_str(&formatted);
}

/// apply formatting based on the tag type
pub fn apply_tag_formatting(tag: &str, param: Option<&str>, content: &str) -> String {
    match tag {
        "b" => content.bold().to_string(),
        "i" => content.italic().to_string(),
        "u" => content.underline().to_string(),
        "s" => content.strikethrough().to_string(),
        "sup" => convert_script(content, ScriptMode::Superscript),
        "sub" => convert_script(content, ScriptMode::Subscript),
        "spoiler" => format!(
            "{}{}{}",
            "░".black().on_black(),
            content.black().on_black(),
            "░".black().on_black()
        ),
        "color" => {
            if let Some(color_str) = param
                && let Some(color) = parse_color(color_str)
            {
                return content.color(color).to_string();
            }
            content.to_string()
        }
        "quote" => content
            .lines()
            .map(|line| format!("{} {}", "│".bright_black(), line))
            .collect::<Vec<_>>()
            .join("\n"),
        "code" => content.on_bright_black().to_string(),
        "section" => {
            let title = param.unwrap_or("Section");
            format!("┌─ {}\n{}\n└─", title.bright_cyan(), content)
        }
        _ => content.to_string(),
    }
}

/// parse content until a closing tag
pub fn parse_until(chars: &mut Peekable<Chars>, closing_tag: &str) -> String {
    mkstr!(buffer);
    let tag_len = closing_tag.len();

    while chars.peek().is_some() {
        let mut lookahead = chars.clone();
        let mut possible_tag = String::with_capacity(tag_len);

        for _ in 0..tag_len {
            if let Some(ch) = lookahead.next() {
                possible_tag.push(ch);
            } else {
                break;
            }
        }

        if possible_tag.eq_ignore_ascii_case(closing_tag) {
            for _ in 0..tag_len {
                chars.next();
            }
            break;
        }

        if let Some(ch) = chars.next() {
            buffer.push(ch);
        }
    }

    let mut inner_chars = buffer.chars().peekable();
    parse(&mut inner_chars)
}

/// handle inline code blocks
pub fn handle_inline_code(chars: &mut Peekable<Chars>, result: &mut String) {
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

/// handle hyperlinks
pub fn handle_link(chars: &mut Peekable<Chars>, result: &mut String) {
    chars.next();

    mkstr!(text);
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
        let url = extract_url(chars);

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

/// extract a url from the char stream
pub fn extract_url(chars: &mut Peekable<Chars>) -> String {
    mkstr!(url);

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

    url
}

/// handles search syntax
pub fn handle_search(chars: &mut Peekable<Chars>, result: &mut String) {
    chars.skip_n(2);

    mkstr!(search);
    let mut depth = 0;

    while let Some(&ch) = chars.peek() {
        if matches_ahead(chars, "{{") {
            depth += 1;
            search.push_str("{{");
            chars.skip_n(2);
        } else if matches_ahead(chars, "}}") {
            if depth == 0 {
                chars.skip_n(2);
                break;
            } else {
                depth -= 1;
                search.push_str("}}");
                chars.skip_n(2);
            }
        } else {
            search.push(ch);
            chars.next();
        }
    }

    result.push_str(&format!("[{}]", search.bright_blue().underline()));
}

/// handles headers
pub fn handle_header(chars: &mut Peekable<Chars>, result: &mut String) {
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

/// handle list items
pub fn handle_list(chars: &mut Peekable<Chars>, result: &mut String) {
    let mut level: usize = 0;

    while chars.peek() == Some(&'*') {
        level += 1;
        chars.next();
    }

    if chars.peek() == Some(&' ') {
        chars.next();
    }

    let indent = " ".repeat(level.saturating_sub(1));
    let bullet = if level == 1 { "•" } else { "◦" };

    result.push_str(&format!("{}{} ", indent, bullet.bright_black()));
}
