use {
    crate::data::tags::TagDatabase,
    inquire::{
        CustomUserError,
        autocompletion::{Autocomplete, Replacement},
    },
    owo_colors::OwoColorize,
    std::sync::Arc,
};

#[derive(Clone)]
pub struct TagAutocompleter {
    tag_db: Arc<TagDatabase>,
    limit: usize,
}

impl TagAutocompleter {
    pub fn new(tag_db: Arc<TagDatabase>) -> Self {
        Self { tag_db, limit: 10 }
    }

    pub fn with_limit(tag_db: Arc<TagDatabase>, limit: usize) -> Self {
        Self { tag_db, limit }
    }

    fn get_current_tag(input: &str) -> &str {
        input.split_whitespace().last().unwrap_or("")
    }

    fn get_prefix(input: &str) -> String {
        if let Some(last_space_idx) = input.rfind(char::is_whitespace) {
            input[..=last_space_idx].to_string()
        } else {
            String::new()
        }
    }

    fn format_suggestion(&self, tag: &str) -> String {
        let canonical = self.tag_db.resolve_alias(tag);

        if canonical != tag {
            format!(
                "{} {} {}",
                tag.cyan(),
                "→".bright_black(),
                canonical.bright_green()
            )
        } else {
            tag.bright_white().to_string()
        }
    }

    fn extract_tag_from_suggestion(suggestion: &str) -> String {
        let stripped = strip_ansi_escapes::strip_str(suggestion);

        let cleaned = stripped
            .trim_start_matches('-')
            .trim_start_matches('~')
            .trim_start_matches('+');

        if let Some(arrow_pos) = cleaned.find(" → ") {
            cleaned[..arrow_pos].to_string()
        } else {
            cleaned.to_string()
        }
    }
}

impl Autocomplete for TagAutocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, CustomUserError> {
        let current_tag = Self::get_current_tag(input);

        let (prefix_char, search_tag) = if let Some(stripped) = current_tag.strip_prefix('-') {
            ("-", stripped)
        } else if let Some(stripped) = current_tag.strip_prefix('~') {
            ("~", stripped)
        } else if let Some(stripped) = current_tag.strip_prefix('+') {
            ("+", stripped)
        } else {
            ("", current_tag)
        };

        if search_tag.is_empty() {
            return Ok(Vec::new());
        }

        let matches = self.tag_db.autocomplete(search_tag, self.limit);

        let formatted: Vec<String> = matches
            .into_iter()
            .map(|tag| {
                let formatted = self.format_suggestion(&tag);

                match prefix_char {
                    "-" => format!("{}{}", "-".red().bold(), formatted),
                    "~" => format!("{}{}", "~".yellow().bold(), formatted),
                    "+" => format!("{}{}", "+".green().bold(), formatted),
                    _ => formatted,
                }
            })
            .collect();

        Ok(formatted)
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, CustomUserError> {
        if let Some(suggestion) = highlighted_suggestion {
            let prefix = Self::get_prefix(input);
            let current_tag = Self::get_current_tag(input);

            let prefix_char = if current_tag.starts_with('-') {
                "-"
            } else if current_tag.starts_with('~') {
                "~"
            } else if current_tag.starts_with('+') {
                "+"
            } else {
                ""
            };

            let tag_name = Self::extract_tag_from_suggestion(&suggestion);

            let canonical_tag = self.tag_db.resolve_alias(&tag_name);

            let new_input = format!("{}{}{} ", prefix, prefix_char, canonical_tag);

            return Ok(Some(new_input));
        }

        let current_tag = Self::get_current_tag(input);

        let (prefix_char, search_tag) = if let Some(stripped) = current_tag.strip_prefix('-') {
            ("-", stripped)
        } else if let Some(stripped) = current_tag.strip_prefix('~') {
            ("~", stripped)
        } else if let Some(stripped) = current_tag.strip_prefix('+') {
            ("+", stripped)
        } else {
            ("", current_tag)
        };

        if search_tag.is_empty() {
            return Ok(None);
        }

        if self.tag_db.exists(search_tag) {
            let prefix = Self::get_prefix(input);
            let canonical_tag = self.tag_db.resolve_alias(search_tag);
            let new_input = format!("{}{}{} ", prefix, prefix_char, canonical_tag);
            return Ok(Some(new_input));
        }

        let matches = self.tag_db.autocomplete(search_tag, self.limit);

        if matches.is_empty() {
            return Ok(None);
        }

        if matches.len() == 1 {
            let prefix = Self::get_prefix(input);
            let canonical_tag = self.tag_db.resolve_alias(&matches[0]);
            let new_input = format!("{}{}{} ", prefix, prefix_char, canonical_tag);
            return Ok(Some(new_input));
        }

        let common_prefix = find_common_prefix(&matches);

        if common_prefix.len() > search_tag.len() {
            let prefix = Self::get_prefix(input);
            let new_input = format!("{}{}{}", prefix, prefix_char, common_prefix);
            return Ok(Some(new_input));
        }

        Ok(None)
    }
}

fn find_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }

    if strings.len() == 1 {
        return strings[0].clone();
    }

    let mut prefix = strings[0].clone();

    for s in &strings[1..] {
        while !s.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return String::new();
            }
        }
    }

    prefix
}
