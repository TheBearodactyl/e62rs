//! autocompleters for demand
use std::sync::Arc;

use {demand::Autocomplete, owo_colors::OwoColorize};

use crate::data::{pools::PoolDb, tags::TagDb};

/// the prefix of an inputted tag
#[derive(Debug, Clone, Copy, PartialEq)]
enum PrefixChar {
    /// no prefix
    None,
    /// exclude (-)
    Exclude,
    /// wildcard (~)
    Wildcard,
    /// include (+)
    Include,
}

impl PrefixChar {
    #[inline]
    /// find the prefix symbol of a string
    fn from_str(s: &str) -> (Self, &str) {
        match s.as_bytes().first() {
            Some(b'-') => (Self::Exclude, &s[1..]),
            Some(b'~') => (Self::Wildcard, &s[1..]),
            Some(b'+') => (Self::Include, &s[1..]),
            _ => (Self::None, s),
        }
    }

    #[inline]
    /// converts self to a string
    fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Exclude => "-",
            Self::Wildcard => "~",
            Self::Include => "+",
        }
    }

    /// colors text based on the given prefix
    fn apply_color(self, formatted: String) -> String {
        match self {
            Self::Exclude => format!("{}{}", "-".red().bold(), formatted),
            Self::Wildcard => format!("{}{}", "~".yellow().bold(), formatted),
            Self::Include => format!("{}{}", "+".green().bold(), formatted),
            Self::None => formatted,
        }
    }
}

#[inline]
/// extract the last token from the given input
fn get_current_token(input: &str) -> &str {
    input.rsplit(char::is_whitespace).next().unwrap_or("")
}

#[inline]
/// get the prefix of the given token
fn get_prefix(input: &str) -> &str {
    match input.rfind(char::is_whitespace) {
        Some(idx) => &input[..=idx],
        None => "",
    }
}

#[inline]
/// strip ansi from a str
fn strip_ansi(s: &str) -> String {
    strip_ansi_escapes::strip_str(s)
}

/// extract tag/pool name from a formatted suggestion
fn extract_name_from_suggestion(suggestion: &str) -> String {
    let stripped = strip_ansi(suggestion);
    let cleaned = stripped.trim_start_matches(&['-', '~', '+'][..]);

    if let Some(arrow_pos) = cleaned.find(" → ") {
        cleaned[..arrow_pos].to_string()
    } else {
        cleaned.to_string()
    }
}

/// an autocompleter for a db
trait AutocompleteDatabase {
    /// provide autocompletions based on a query
    fn autocomplete(&self, query: &str, limit: usize) -> Vec<String>;
    /// resolve an entry based on the name
    fn resolve_name(&self, name: &str) -> String;
    /// format an entry for display
    fn format_entry(&self, name: &str) -> String;
}

/// a generic autocompleter
struct GenericAutocompleter<T: AutocompleteDatabase> {
    /// the db
    db: Arc<T>,
    /// the limit of autocompletions to display at a tme
    limit: usize,
}

impl<T: AutocompleteDatabase> GenericAutocompleter<T> {
    /// make a new autocompleter
    fn new(db: Arc<T>, limit: usize) -> Self {
        Self { db, limit }
    }

    /// get suggestions based on input
    fn get_suggestions_impl(&self, input: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let current_token = get_current_token(input);
        let (prefix_char, search_query) = PrefixChar::from_str(current_token);

        if search_query.is_empty() {
            return Ok(Vec::new());
        }

        let matches = self.db.autocomplete(search_query, self.limit);
        let formatted: Vec<String> = matches
            .into_iter()
            .map(|name| {
                let formatted = self.db.format_entry(&name);
                prefix_char.apply_color(formatted)
            })
            .collect();

        Ok(formatted)
    }

    /// get completions based on input
    fn get_completion_impl(
        &self,
        input: &str,
        highlighted_suggestion: Option<&str>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let Some(suggestion) = highlighted_suggestion else {
            return Ok(None);
        };

        let prefix = get_prefix(input);
        let current_token = get_current_token(input);
        let (prefix_char, _) = PrefixChar::from_str(current_token);
        let name = extract_name_from_suggestion(suggestion);
        let canonical_name = self.db.resolve_name(name.as_str());
        let mut completion = String::with_capacity(
            prefix.len() + prefix_char.as_str().len() + canonical_name.len() + 1,
        );

        completion.push_str(prefix);
        completion.push_str(prefix_char.as_str());
        completion.push_str(&canonical_name);
        completion.push(' ');

        Ok(Some(completion))
    }
}

impl AutocompleteDatabase for TagDb {
    fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        self.autocomplete(query, limit)
    }

    fn resolve_name(&self, name: &str) -> String {
        self.resolve_alias(name)
    }

    fn format_entry(&self, name: &str) -> String {
        let canonical = self.resolve_alias(name);

        if canonical != name {
            format!(
                "{} {} {}",
                name.cyan(),
                "→".bright_black(),
                canonical.bright_green()
            )
        } else {
            name.bright_white().to_string()
        }
    }
}

#[derive(Clone)]
/// a tag autocompleter
pub struct TagAutocompleter {
    /// the inner generic completer
    inner: Arc<GenericAutocompleter<TagDb>>,
}

impl TagAutocompleter {
    /// make a new tag autocompleter
    pub fn new(tag_db: Arc<TagDb>) -> Self {
        Self::with_limit(tag_db, 10)
    }

    /// make a new tag autocompleter with the given tag limit
    pub fn with_limit(tag_db: Arc<TagDb>, limit: usize) -> Self {
        Self {
            inner: Arc::new(GenericAutocompleter::new(tag_db, limit)),
        }
    }
}

impl Autocomplete for TagAutocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.inner.get_suggestions_impl(input)
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<&str>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.inner
            .get_completion_impl(input, highlighted_suggestion)
    }
}

impl AutocompleteDatabase for PoolDb {
    fn autocomplete(&self, query: &str, limit: usize) -> Vec<String> {
        self.autocomplete(query, limit)
    }

    fn resolve_name(&self, name: &str) -> String {
        name.to_string()
    }

    fn format_entry(&self, name: &str) -> String {
        name.bright_cyan().to_string()
    }
}

#[derive(Clone)]
/// a pool autocompleter
pub struct PoolAutocompleter {
    /// the inner generic completer for the pools db
    inner: Arc<GenericAutocompleter<PoolDb>>,
}

impl PoolAutocompleter {
    /// make a new pool autocompleter
    pub fn new(pool_db: Arc<PoolDb>) -> Self {
        Self::with_limit(pool_db, 10)
    }

    /// make a new pool autocompleter with a given result limit
    pub fn with_limit(pool_db: Arc<PoolDb>, limit: usize) -> Self {
        Self {
            inner: Arc::new(GenericAutocompleter::new(pool_db, limit)),
        }
    }
}

impl Autocomplete for PoolAutocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.inner.get_suggestions_impl(input)
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<&str>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        self.inner
            .get_completion_impl(input, highlighted_suggestion)
    }
}
