//! autocompleters for bearask
use {
    crate::data::{pools::PoolDb, tags::TagDb},
    bearask::{Autocomplete, Replacement},
    owo_colors::OwoColorize,
    std::sync::Arc,
};

/// the prefix of an inputted tag
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrefixChar {
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
    #[bearive::argdoc]
    pub fn find_from_str(
        /// the tag string
        s: &str,
    ) -> (Self, &str) {
        match s.as_bytes().first() {
            Some(b'-') => (Self::Exclude, &s[1..]),
            Some(b'~') => (Self::Wildcard, &s[1..]),
            Some(b'+') => (Self::Include, &s[1..]),
            _ => (Self::None, s),
        }
    }

    #[inline]
    /// converts self to a string
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Exclude => "-",
            Self::Wildcard => "~",
            Self::Include => "+",
        }
    }

    /// colors text based on the given prefix
    ///
    /// # Arguments
    ///
    /// * `formatted` - the formatted string
    pub fn apply_color(self, formatted: String) -> String {
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
///
/// # Arguments
///
/// * `input` - the string to extract from
pub fn get_current_token(input: &str) -> &str {
    input.rsplit(char::is_whitespace).next().unwrap_or("")
}

#[inline]
/// get the prefix of the given token
///
/// # Arguments
///
/// * `input` - the string to get the prefix of
pub fn get_prefix(input: &str) -> &str {
    match input.rfind(char::is_whitespace) {
        Some(idx) => &input[..=idx],
        None => "",
    }
}

#[inline]
/// strip ansi from a str
///
/// # Arguments
///
/// * `s` - the string to strip
pub fn strip_ansi(s: &str) -> String {
    strip_ansi_escapes::strip_str(s)
}

/// extract tag/pool name from a formatted suggestion
///
/// # Arguments
///
/// * `suggestion` - the suggestion string to extract from
pub fn extract_name_from_suggestion(suggestion: &str) -> String {
    let stripped = strip_ansi(suggestion);
    let cleaned = stripped.trim_start_matches(&['-', '~', '+'][..]);

    if let Some(arrow_pos) = cleaned.find(" → ") {
        cleaned[..arrow_pos].to_string()
    } else {
        cleaned.to_string()
    }
}

/// an autocompleter for a db
pub trait AutocompleteDatabase {
    /// provide autocompletions based on a query
    ///
    /// # Arguments
    ///
    /// * `query` - the query to base completions off of
    /// * `limit` - the max amount of completions to provide
    fn autocomplete(&self, query: &str, limit: usize) -> Vec<String>;

    /// resolve an entry based on the name
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the entry
    fn resolve_name(&self, name: &str) -> String;

    /// format an entry for display
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the entry
    fn format_entry(&self, name: &str) -> String;
}

/// a generic autocompleter
pub struct GenericAutocompleter<T: AutocompleteDatabase> {
    /// the db
    pub db: Arc<T>,
    /// the limit of autocompletions to display at a tme
    pub limit: usize,
}

impl<T: AutocompleteDatabase> GenericAutocompleter<T> {
    /// make a new autocompleter
    ///
    /// # Arguments
    ///
    /// * `db` - the database to derive completions from
    /// * `limit` - the max amount of completions to show at a time
    pub fn new(db: Arc<T>, limit: usize) -> Self {
        Self { db, limit }
    }

    /// get suggestions based on input
    ///
    /// # Arguments
    ///
    /// * `input` - the input to base suggestions off of
    pub fn get_suggestions_impl(&self, input: &str) -> Vec<String> {
        let current_token = get_current_token(input);
        let (prefix_char, search_query) = PrefixChar::find_from_str(current_token);

        if search_query.is_empty() {
            return Vec::new();
        }

        let matches = self.db.autocomplete(search_query, self.limit);
        let formatted: Vec<String> = matches
            .into_iter()
            .map(|name| {
                let formatted = self.db.format_entry(&name);
                prefix_char.apply_color(formatted)
            })
            .collect();

        formatted
    }

    /// get completions based on input
    ///
    /// # Arguments
    ///
    /// * `input` - the input to base suggestions off of
    /// * `highlighted_suggestion` - the current highlighted suggestion
    pub fn get_completion_impl(
        &self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Option<String> {
        let suggestion = highlighted_suggestion?;

        let prefix = get_prefix(input);
        let current_token = get_current_token(input);
        let (prefix_char, _) = PrefixChar::find_from_str(current_token);
        let name = extract_name_from_suggestion(&suggestion);
        let canonical_name = self.db.resolve_name(name.as_str());
        let mut completion = String::with_capacity(
            prefix.len() + prefix_char.as_str().len() + canonical_name.len() + 1,
        );

        completion.push_str(prefix);
        completion.push_str(prefix_char.as_str());
        completion.push_str(&canonical_name);
        completion.push(' ');

        Some(completion)
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
    pub inner: Arc<GenericAutocompleter<TagDb>>,
}

impl TagAutocompleter {
    /// make a new tag autocompleter
    ///
    /// # Arguments
    ///
    /// * `tag_db` - a loaded tag database
    pub fn new(tag_db: Arc<TagDb>) -> Self {
        Self::with_limit(tag_db, 10)
    }

    /// make a new tag autocompleter with the given tag limit
    ///
    /// # Arguments
    ///
    /// * `tag_db` - a loaded tag database
    /// * `limit` - the limit of completions to show at a time
    pub fn with_limit(tag_db: Arc<TagDb>, limit: usize) -> Self {
        Self {
            inner: Arc::new(GenericAutocompleter::new(tag_db, limit)),
        }
    }
}

impl Autocomplete for TagAutocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, String> {
        Ok(self.inner.get_suggestions_impl(input))
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, String> {
        Ok(self
            .inner
            .get_completion_impl(input, highlighted_suggestion))
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
    pub inner: Arc<GenericAutocompleter<PoolDb>>,
}

impl PoolAutocompleter {
    /// make a new pool autocompleter
    ///
    /// # Arguments
    ///
    /// * `pool_db` - a loaded pool database
    pub fn new(pool_db: Arc<PoolDb>) -> Self {
        Self::with_limit(pool_db, 10)
    }

    /// make a new pool autocompleter with a given result limit
    ///
    /// # Arguments
    ///
    /// * `pool_db` - a loaded pool database
    /// * `limit` - the limit of completions to show at a time
    pub fn with_limit(pool_db: Arc<PoolDb>, limit: usize) -> Self {
        Self {
            inner: Arc::new(GenericAutocompleter::new(pool_db, limit)),
        }
    }
}

impl Autocomplete for PoolAutocompleter {
    fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, String> {
        Ok(self.inner.get_suggestions_impl(input))
    }

    fn get_completion(
        &mut self,
        input: &str,
        highlighted_suggestion: Option<String>,
    ) -> Result<Replacement, String> {
        Ok(self
            .inner
            .get_completion_impl(input, highlighted_suggestion))
    }
}
