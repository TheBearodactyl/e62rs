//! parser for output file format
use {
    color_eyre::{Result, eyre::Context},
    hashbrown::HashMap,
    rand::seq::IndexedRandom,
};

/// a parsed format template
#[derive(Clone, Debug)]
pub struct FormatTemplate {
    /// all parts of the parsed template
    parts: Vec<FormatPart>,
}

/// a part of the format template
#[derive(Clone, Debug)]
enum FormatPart {
    /// literal text
    Literal(String),
    /// a placeholder
    Placeholder(Placeholder),
}

/// a placeholder in the format str
#[derive(Clone, Debug)]
pub struct Placeholder {
    /// the key name
    pub key: String,
    /// opitonal index/range spec
    pub idx: Option<IndexSpec>,
}

/// index spec for array-esque placeholders
#[derive(Clone, Debug)]
pub struct IndexSpec {
    /// the selection type
    pub selection: IndexSelection,
    /// whether to reverse the result
    pub reverse: bool,
}

/// type of index selection
#[derive(Clone, Debug)]
pub enum IndexSelection {
    /// first n items (e.g. $tags\[5\])
    First(usize),
    /// range from items l to r (e.g. $tags\[2..5\])
    Range(usize, usize),
    /// from n onwards (e.g. $tags\[1..\])
    From(usize),
    /// up to n (e.g. $tags\[..3\])
    To(usize),
    /// last n items (e.g. $tags\[l5\])
    Last(usize),
    /// random n items (e.g. $tags\[r3\])
    Random(usize),
}

impl FormatTemplate {
    /// parse a format str into a template
    pub fn parse(format_str: &str) -> color_eyre::Result<Self> {
        let mut parts = Vec::new();
        let mut chars = format_str.chars().peekable();
        let mut curlit = String::new();

        while let Some(ch) = chars.next() {
            if ch == '$' {
                if !curlit.is_empty() {
                    parts.push(FormatPart::Literal(curlit.clone()));
                    curlit.clear();
                }

                let placeholder =
                    Self::parse_placeholder(&mut chars).wrap_err("Failed to parse placeholder")?;

                parts.push(FormatPart::Placeholder(placeholder));
            } else if ch == '\\' {
                if let Some(next_ch) = chars.next() {
                    curlit.push(next_ch);
                } else {
                    curlit.push(ch);
                }
            } else {
                curlit.push(ch);
            }
        }

        if !curlit.is_empty() {
            parts.push(FormatPart::Literal(curlit));
        }

        Ok(Self { parts })
    }

    /// parse a single placeholder
    fn parse_placeholder(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<Placeholder> {
        let mut key = String::new();
        let mut in_brackets = false;
        let mut bracket_content = String::new();

        while let Some(&ch) = chars.peek() {
            if ch == '[' {
                in_brackets = true;
                chars.next();
                break;
            } else if ch.is_alphanumeric() || ch == '_' {
                key.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        if key.is_empty() {
            color_eyre::eyre::bail!("Empty placeholder key");
        }

        let idx = if in_brackets {
            for ch in chars.by_ref() {
                if ch == ']' {
                    break;
                }

                bracket_content.push(ch);
            }

            Some(
                Self::parse_index_spec(&bracket_content)
                    .wrap_err_with(|| format!("Invalid index spec: {}", bracket_content))?,
            )
        } else {
            None
        };

        Ok(Placeholder { key, idx })
    }

    /// parse an index spec
    fn parse_index_spec(spec: &str) -> Result<IndexSpec> {
        let spec = spec.trim();

        let (spec, reverse) = if !spec.ends_with('r') {
            (spec, false)
        } else {
            (&spec[..spec.len() - 1], true)
        };

        let selection = if let Some(stripped) = spec.strip_prefix('l') {
            let count = stripped
                .parse::<usize>()
                .wrap_err("invalid last count spec")?;
            IndexSelection::Last(count)
        } else if let Some(stripped) = spec.strip_prefix('r') {
            let count = stripped
                .parse::<usize>()
                .wrap_err("Invalid random count spec")?;
            IndexSelection::Random(count)
        } else if spec.contains("..") {
            let parts: Vec<&str> = spec.split("..").collect();
            if parts.len() != 2 {
                color_eyre::eyre::bail!("Invalid range spec: {}", spec);
            }

            match (parts[0].trim(), parts[1].trim()) {
                ("", "") => {
                    color_eyre::eyre::bail!("Invalid range spec: must provide at least one bound")
                }

                ("", end) => {
                    let end_idx = end.parse::<usize>().wrap_err("Invalid end index")?;
                    IndexSelection::To(end_idx)
                }

                (start, "") => {
                    let start_idx = start.parse::<usize>().wrap_err("Invalid start index")?;
                    IndexSelection::From(start_idx)
                }

                (start, end) => {
                    let start_idx = start.parse::<usize>().wrap_err("Invalid start index")?;
                    let end_idx = end.parse::<usize>().wrap_err("Invalid end index")?;

                    if start_idx >= end_idx {
                        color_eyre::eyre::bail!("Start index must be less than end index");
                    }

                    IndexSelection::Range(start_idx, end_idx)
                }
            }
        } else {
            let count = spec.parse::<usize>().wrap_err("Invalid count spec")?;
            IndexSelection::First(count)
        };

        Ok(IndexSpec { selection, reverse })
    }

    /// render the current template with the given context
    pub fn render(&self, context: &HashMap<String, String>) -> Result<String> {
        let mut result = String::new();

        for part in &self.parts {
            match part {
                FormatPart::Literal(text) => result.push_str(text),
                FormatPart::Placeholder(placeholder) => {
                    let val = self
                        .resolve_placeholder(placeholder, context)
                        .wrap_err_with(|| {
                            format!("Failed to resolve placeholder: {}", placeholder.key)
                        })?;

                    result.push_str(&val);
                }
            }
        }

        Ok(result)
    }

    /// render with array context for indexed placeholders
    pub fn render_with_arrays(
        &self,
        simple_context: &HashMap<String, String>,
        arr_context: &HashMap<String, Vec<String>>,
    ) -> Result<String> {
        let mut result = String::new();

        for part in &self.parts {
            match part {
                FormatPart::Literal(text) => result.push_str(text),
                FormatPart::Placeholder(p) => {
                    let value = if let Some(index) = &p.idx {
                        if let Some(arr) = arr_context.get(&p.key) {
                            self.apply_index_spec(arr, index)
                        } else {
                            color_eyre::eyre::bail!(
                                "Array placeholder '{}' not found in context",
                                p.key
                            );
                        }
                    } else {
                        simple_context
                            .get(&p.key)
                            .cloned()
                            .or_else(|| arr_context.get(&p.key).map(|arr| arr.join(", ")))
                            .ok_or_else(|| {
                                color_eyre::eyre::eyre!(
                                    "Placeholder '{}' not found in context",
                                    p.key
                                )
                            })?
                    };

                    result.push_str(&value);
                }
            }
        }

        Ok(result)
    }

    /// resolve a placeholder value
    fn resolve_placeholder(
        &self,
        placeholder: &Placeholder,
        context: &HashMap<String, String>,
    ) -> Result<String> {
        context.get(&placeholder.key).cloned().ok_or_else(|| {
            color_eyre::eyre::eyre!("Placeholder '{}' not found in context", placeholder.key)
        })
    }

    /// apply index spec to an array
    fn apply_index_spec(&self, array: &[String], spec: &IndexSpec) -> String {
        let mut items: Vec<&String> = match &spec.selection {
            IndexSelection::First(n) => array.iter().take(*n).collect(),
            IndexSelection::Range(start, end) => {
                array.iter().skip(*start).take(end - start).collect()
            }
            IndexSelection::From(start) => array.iter().skip(*start).collect(),
            IndexSelection::To(end) => array.iter().take(*end).collect(),
            IndexSelection::Last(n) => {
                let start = array.len().saturating_sub(*n);
                array.iter().skip(start).collect()
            }
            IndexSelection::Random(n) => {
                let mut rng = rand::rng();
                let count = (*n).min(array.len());
                array.choose_multiple(&mut rng, count).collect()
            }
        };

        if spec.reverse {
            items.reverse();
        }

        items
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// get all placeholder keys in the template
    pub fn get_placeholders(&self) -> Vec<String> {
        self.parts
            .iter()
            .filter_map(|part| {
                if let FormatPart::Placeholder(p) = part {
                    Some(p.key.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
