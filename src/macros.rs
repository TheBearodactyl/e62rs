//! macros used by e62rs

/// format a value (lol)
#[macro_export]
macro_rules! fmt_value {
    () => {
        |v| format!("{}", v)
    };
    (debug) => {
        |v| format!("{:?}", v)
    };
}

/// implement Send + Sync for a type
#[macro_export]
macro_rules! sendsync {
    ($ty:ty) => {
        unsafe impl<T: $crate::data::Entry> Send for $ty {}
        unsafe impl<T: $crate::data::Entry> Sync for $ty {}
    };
}

/// repeat an expression n times
#[macro_export]
macro_rules! repeat {
    ($n:expr, $body:expr) => {
        for _ in 0..$n {
            $body;
        }
    };
}

/// make a new `String`
#[macro_export]
macro_rules! mkstr {
    ($n:ident) => {
        let mut $n = String::new();
    };

    ($n:ident, $capacity:expr) => {
        let mut $n = String::with_capacity($capacity);
    };
}

/// make a new `Vec`
#[macro_export]
macro_rules! mkvec {
    ($n:ident, $t:ty) => {
        let mut $n: Vec<$t> = Vec::new();
    };

    ($n:ident, $t:ty, $c:expr) => {
        let mut $n: Vec<$t> = Vec::with_capacity($c);
    };
}

/// if an option is enabled, perform an expression
///
/// # Examples
///
/// ```
/// use e62rs::opt_and;
///
/// fn save_metadata() {
///     println!("saved metadata");
/// }
///
/// opt_and!(download.save_metadata, save_metadata());
/// ```
#[macro_export]
macro_rules! opt_and {
    ($field:ident, $a:expr) => {
        if $crate::getopt!($field) {
            $a
        }
    };

    ($lvl1:ident . $field:ident, $a:expr) => {
        if $crate::getopt!($lvl1.$field) {
            $a
        }
    };
}

/// run a block of code until it succeeds
#[macro_export]
macro_rules! retry {
    (
        retries: $retries:expr,
        delay: $delay_ms:expr,
        code: $code:expr
    ) => {{
        let mut attempts = 0;
        let max_retries = $retries;
        let delay = std::time::Duration::from_millis($delay_ms);
        let mut last_error;

        loop {
            attempts += 1;

            match $code {
                Ok(value) => break Ok(value),
                Err(e) => {
                    last_error = Some(e);

                    if attempts >= max_retries {
                        break Err(last_error.unwrap());
                    }

                    std::thread::sleep(delay);
                }
            }
        }
    }};
}

/// make a menu
#[macro_export]
macro_rules! menu {
    (
        $(#[$enum_meta:meta])*
        $vis:vis $enum_name:ident { filterable: $filterable:expr,
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => {
                    label: {
                        english => $label:expr,
                        japanese => $japanese_label:expr,
                        spanish => $spanish_label:expr
                    },
                    desc: {
                        english => $desc:expr,
                        japanese => $japanese_desc:expr,
                        spanish => $spanish_desc:expr
                    },
                    online: $online:expr
                }
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $enum_name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl ::std::fmt::Display for $enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    $(
                        Self::$variant => write!(f, "{}", $desc),
                    )*
                }
            }
        }

        impl $enum_name {
            /// get stats on translation progress for each language
            pub fn translation_stats() -> hashbrown::HashMap<&'static str, $crate::ui::menus::TranslationStats> {
                let mut stats = hashbrown::HashMap::new();
                let languages = ["japanese", "spanish", "english"];

                for lang in languages {
                    stats.insert(lang, TranslationStats::default());
                }

                $(
                    stats.get_mut("english").unwrap().total_variants += 1;
                    stats.get_mut("spanish").unwrap().total_variants += 1;
                    stats.get_mut("japanese").unwrap().total_variants += 1;

                    if !$label.is_empty() {
                        stats.get_mut("english").unwrap().labels_translated += 1;
                    }

                    if !$desc.is_empty() {
                        stats.get_mut("english").unwrap().descriptions_translated += 1;
                    }

                    if !$japanese_label.is_empty() {
                        stats.get_mut("japanese").unwrap().labels_translated += 1;
                    }

                    if !$japanese_desc.is_empty() {
                        stats.get_mut("japanese").unwrap().descriptions_translated += 1;
                    }

                    if !$spanish_label.is_empty() {
                        stats.get_mut("spanish").unwrap().labels_translated += 1;
                    }

                    if !$spanish_desc.is_empty() {
                        stats.get_mut("spanish").unwrap().descriptions_translated += 1;
                    }
                )*

                stats
            }

            /// if s is empty, return d, else return s
            #[inline]
            const fn select_translation(s: &'static str, d: &'static str) -> &'static str {
                if s.is_empty() {
                    d
                } else {
                    s
                }
            }

            /// display a menu and return the selected option
            #[must_use]
            pub fn select(prompt: &str) -> ::demand::Select<'_, Self> {
                use $crate::config::options::Language;

                let mut selection = ::demand::Select::new(prompt)
                    .filterable($filterable)
                    .theme(&ROSE_PINE);

                $(
                    {
                        let lang = $crate::getopt!(language);
                        let has_internet = $crate::utils::check_for_internet();

                        let label = match lang {
                            Language::English => $label,
                            Language::Spanish => Self::select_translation($spanish_label, $label),
                            Language::Japanese => Self::select_translation($japanese_label, $label),
                        };

                        let mut description = match lang {
                            Language::English => ::std::string::String::from($desc),
                            Language::Spanish => ::std::string::String::from(
                                Self::select_translation($spanish_desc, $desc)
                            ),
                            Language::Japanese => ::std::string::String::from(
                                Self::select_translation($japanese_desc, $desc)
                            ),
                        };

                        if has_internet == $online {
                            use ::owo_colors::OwoColorize;
                            let warning = match lang {
                                Language::English => " (REQUIRES INTERNET ACCESS)",
                                Language::Spanish => " (SE REQUIERE ACCESO A INTERNET)",
                                Language::Japanese => " (<wip>)",
                            };

                            description.push_str(&warning.red().to_string());
                        }

                        selection = selection.option(
                            ::demand::DemandOption::new(Self::$variant)
                                .label(label)
                                .description(&description)
                        );
                    }
                )*

                selection
            }

            /// get the label of the given variant
            #[allow(dead_code)]
            pub const fn label(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $label,
                    )*
                }
            }
        }
    };
}

/// implement display for a type
#[macro_export]
macro_rules! impl_display {
    ($type:ty, $name:expr, $color:ident, $($field:ident: $format:expr),*) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                writeln!(f, "{} {{", $name.$color())?;
                $(
                    writeln!(f, "  {}: {}", stringify!($field).yellow(), $format(&self.$field))?;
                )*
                writeln!(f, "}}")
            }
        }
    };
}

/// make a theme
#[macro_export]
macro_rules! impl_theme {
    ($name:ident, $display_name:expr, $variant:expr, $colors:expr) => {
        /// a theme preset
        #[derive(Clone, Default)]
        pub struct $name;

        impl Theme for $name {
            fn colors() -> ThemeColors {
                $colors
            }

            fn name() -> &'static str {
                $display_name
            }

            fn variant() -> ThemeVariant {
                $variant
            }
        }
    };
}

/// helper macro for generating validators
#[macro_export]
macro_rules! validator {
    ($struct_name:ty, $( $field:ident => $requirement:expr, $err_msg:expr );* $(;)? ) => {
        impl Validate for $struct_name {
            fn validate(&self) -> Result<(), Vec<String>> {
                let mut errors: Vec<String> = Vec::new();

                $(
                    if let Some(ref value) = self.$field {
                        if !($requirement)(value) {
                            errors.push(format!("{}: {}", stringify!($field), $err_msg));
                        }
                    }
                )*

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    };
}

/// helper macro for nested validation
#[macro_export]
macro_rules! validator_nested {
    ($struct_name:ty,
        fields: { $( $field:ident => $requirement:expr, $err_msg:expr );* $(;)? }
        nested: { $( $nested:ident );* $(;)? }
    ) => {
        impl Validate for $struct_name {
            fn validate(&self) -> Result<(), Vec<String>> {
                let mut errors: Vec<String> = Vec::new();

                $(
                    if let Some(ref value) = self.$field {
                        if !($requirement)(value) {
                            errors.push(format!("{}: {}", stringify!($field), $err_msg));
                        }
                    }
                )*

                $(
                    if let Some(ref nested) = self.$nested {
                        if let Err(nested_errors) = nested.validate() {
                            for err in nested_errors {
                                errors.push(format!("{}.{}", stringify!($nested), err));
                            }
                        }
                    }
                )*

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    };
}

/// get the current value of a given setting
#[macro_export]
macro_rules! getopt {
    () => {
        $crate::config::instance::config()
    };

    ($field:ident) => {{
        $crate::config::instance::get_or_default(
            |c| c.$field.clone(),
            $crate::config::options::E62Rs::default()
                .$field
                .expect(concat!("Default value missing for: ", stringify!($field))),
        )
    }};

    ($lvl1:ident . $field:ident) => {{
        $crate::config::instance::get_or_default(
            |c| c.$lvl1.as_ref().and_then(|sub| sub.$field.clone()),
            $crate::config::options::E62Rs::default()
                .$lvl1
                .and_then(|sub| sub.$field)
                .expect(concat!(
                    "Default value missing for: ",
                    stringify!($lvl1),
                    ".",
                    stringify!($field)
                )),
        )
    }};

    ($lvl1:ident . $lvl2:ident . $field:ident) => {{
        $crate::config::instance::get_or_default(
            |c| {
                c.$lvl1
                    .as_ref()
                    .and_then(|sub| sub.$lvl2.as_ref())
                    .and_then(|sub| sub.$field.clone())
            },
            $crate::config::options::E62Rs::default()
                .$lvl1
                .and_then(|sub| sub.$lvl2)
                .and_then(|sub| sub.$field)
                .expect(concat!(
                    "Default value missing for: ",
                    stringify!($lvl1),
                    ".",
                    stringify!($lvl2),
                    ".",
                    stringify!($field)
                )),
        )
    }};

    (raw $field:ident) => {{
        $crate::config::instance::config()
            .ok()
            .and_then(|c| c.$field.clone())
    }};

    (raw $lvl1:ident . $field:ident) => {{
        $crate::config::instance::config()
            .ok()
            .and_then(|c| c.$lvl1.as_ref().and_then(|sub| sub.$field.clone()))
    }};

    (raw $lvl1:ident . $lvl2:ident . $field:ident) => {{
        $crate::config::instance::config().ok().and_then(|c| {
            c.$lvl1
                .as_ref()
                .and_then(|sub| sub.$lvl2.as_ref())
                .and_then(|sub| sub.$field.clone())
        })
    }};
}

#[cfg(test)]
mod tests {
    fn unstable_operation(counter: &mut i32) -> color_eyre::Result<String, String> {
        *counter += 1;

        if *counter < 3 {
            println!("attempt {}: failed...", counter);
            Err("failed".to_string())
        } else {
            println!("attempt {}: succeeded...", counter);
            Ok("succeeded".to_string())
        }
    }

    #[test]
    fn test_retry() {
        let mut counter = 0;

        let result = retry!(
            retries: 100,
            delay: 500,
            code: {
                unstable_operation(&mut counter)
            }
        );

        match result {
            Ok(msg) => println!("final result: {}", msg),
            Err(e) => println!("final error: {}", e),
        }
    }
}
