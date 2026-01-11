//! utilities used across e62rs
use {
    crate::getopt,
    base64::{Engine, engine::general_purpose},
    color_eyre::eyre::{Context, Result},
    reqwest::header::{AUTHORIZATION, HeaderMap},
    serde::{Deserialize, Serialize},
    std::{
        fmt::Debug,
        fs::{File, OpenOptions},
        io::{BufWriter, Write},
        path::{Path, PathBuf},
    },
    tracing::Level,
};

/// deserialize into a boolean
///
/// tries to convert the given deserializer into a string, and returns whether or not the
/// deserialized string is equal to `t`, otherwise returning an error
#[bearive::argdoc]
#[error = "returns an error if it fails to deserialize into a string"]
pub fn deserialize_bool_from_str<'de, D>(
    /// the deserializer
    deserializer: D,
) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s == "t")
}

/// deserialize into a list of post ids
///
/// tries to convert the given deserializer into a string, checks for curly braces, then separates
/// the contents of said curly braces into a list of integers using `,` as the delimiter. returns
/// an empty list if the deserialized string isn't surrounded by curly braces
#[bearive::argdoc]
#[error = "returns an error if it fails to deserialize into a string"]
pub fn deserialize_post_ids<'de, D>(
    /// the deserializer
    deserializer: D,
) -> Result<Vec<i64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    if s.starts_with('{') && s.ends_with('}') {
        let inner = &s[1..s.len() - 1];
        if inner.is_empty() {
            return Ok(Vec::new());
        }

        let ids: Result<Vec<i64>, _> = inner
            .split(',')
            .map(|id| id.trim().parse::<i64>())
            .collect();

        ids.map_err(serde::de::Error::custom)
    } else {
        Ok(Vec::new())
    }
}

/// ¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶
///
/// ¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶
/// ¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶¶
macro_rules! 𝒻 {
    ($e:expr) => {
        Some($e.as_bytes())
    };
}

/// validate the base api url
///
/// NjM2ODY1NjM2QjczMjA2OTY2MjA2MTIwNjc2OTc2NjU2RTIwNzM3NDcyNjk2RTY3MjA2MzZGNkU3NDYxNjk2RTczMjA3NDY4NjUyMDcwNjE3NDc0NjU3MjZFMjAyNzYxNjkyNw==
#[allow(clippy::all, non_snake_case, nonstandard_style)]
#[bearive::argdoc]
pub fn ꟿ<T>(
    /// the url to validate
    input: T,
) -> bool
where
    T: AsRef<str>,
{
    type Ӂ = u16;
    type Ӓ = u8;
    type Ӟ = usize;
    const Ӥ: Ӂ = {
        const Ӧ: Ӂ = 0x61;
        const Ӫ: Ӂ = 0x69;
        (Ӧ << (4 << 1)) | Ӫ
    };
    const Ӱ: Ӟ = std::mem::size_of::<()>();
    const Ӳ: Ӟ = !Ӱ as Ӟ;
    const Ӷ: Ӟ = {
        let Ӻ = Ӳ;
        let ӽ = Ӳ;
        (Ӻ ^ ӽ) + Ӳ
    };

    (|ὼ: &dyn Fn(&[Ӓ]) -> bool| match 𝒻!(input.as_ref()) {
        Some(ref ꬷ) if ꬷ.len() >= Ӷ => ὼ(ꬷ),
        _ => (|Ӿ: [[[bool; Ӳ]; Ӳ]; Ӳ]| Ӿ[Ӱ][Ӱ][Ӱ])([[[false; Ӳ]; Ӳ]; Ӳ]),
    })(
        &(|s: &[Ӓ]| {
            s.windows((|ӂ: Ӟ| (|ӄ: Ӟ| ӄ << ӄ)(ӂ))(Ӳ))
                .map(|ő| {
                    (|ӆ: Ӓ, Ӊ: Ӓ| {
                        (|ӌ: Ӂ, Ӎ: Ӓ| ӌ << Ӎ)(ӆ as Ӂ, ((!(!Ӱ)) << (!(!Ӱ))) as Ӓ) | Ӊ as Ӂ
                    })(ő[Ӱ ^ Ӱ], ő[(|Ӕ: Ӟ| !(!Ӕ) as Ӟ)(Ӱ)])
                })
                .fold((|ӗ: Ӟ| Vec::with_capacity(ӗ))(Ӱ), |mut ψ, ϋ| {
                    ψ.push((|ӛ: Ӂ, ӝ: Ӂ| ӛ ^ ӝ)(ϋ, Ӥ));
                    ψ
                })
                .iter()
                .any(|&ϖ| {
                    (|Ӡ: Ӂ| (|ӣ: Ӂ| (|Ӧ: bool| (|ӫ: bool| ӫ)(Ӧ))(ӣ == (Ӱ as Ӂ)))(Ӡ))(ϖ)
                })
        }),
    )
}

/// make an auth header based on the loaded config
///
/// formats the configured username and api-key into a single string and uses that string to create
/// a basic-auth header, then returning the header as an auth header for reqwest
#[bearive::argdoc]
#[error = "returns an error if it fails to convert the created basic auto str to a header value"]
pub fn create_auth_header() -> Result<HeaderMap> {
    let auth_str = format!("{}:{}", getopt!(login.username), getopt!(login.api_key));
    let encoded = general_purpose::STANDARD.encode(&auth_str);
    let auth_value = format!("Basic {}", encoded);
    let mut headers = HeaderMap::new();

    headers.insert(
        AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth_value)?,
    );

    Ok(headers)
}

/// shorten a path to a given length
///
/// takes a path and shortens each component to a given size
#[bearive::argdoc]
pub fn shorten_path(
    /// the path string to shorten
    path: &str,
    /// the max length for each component
    max_len: usize,
) -> String {
    let shortened = Path::new(path)
        .components()
        .map(|component| {
            let s = component.as_os_str().to_string_lossy();
            if s.len() > max_len {
                s[..max_len].to_string()
            } else {
                s.to_string()
            }
        })
        .collect::<PathBuf>()
        .to_string_lossy()
        .to_string();

    shortened.replace("\\", "/")
}

/// create and write to an ADS stream (windows only)
///
/// uses a [`FileWriter`] to open an NTFS alternate data stream on a given file with a specified
/// name, and then write the given data to the created alternate data stream
#[bearive::argdoc]
#[error = "it fails to open an ADS with the specified name"]
pub fn write_to_ads<P, T>(
    /// the path to the file to open an alternate data stream on
    file_path: P,
    /// the name of the new ad stream
    stream_name: &str,
    /// the data to write to the ad stream
    data: T,
) -> Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let mut ads_writer = FileWriter::ads(file_path, stream_name, true)?;
    ads_writer.write(&data)?;

    Ok(())
}

/// check if there's internet access
pub fn check_for_internet() -> bool {
    reqwest::blocking::get(crate::getopt!(http.api)).is_ok()
}

/// write some json data to a given file
#[bearive::argdoc]
#[error = "returns an error if it fails to open `file_path`"]
#[error = "returns an error if it fails to write `data` to `file_path`"]
pub fn write_to_json<P: AsRef<Path>, T: Serialize>(
    /// the path to the json file
    file_path: P,
    /// the data to write to `file_path`
    data: &T,
) -> Result<()> {
    let mut json_writer = FileWriter::json(file_path, true)?;
    json_writer.write(data)?;

    Ok(())
}

/// convert a string to a log level
///
/// takes a given string and converts it into a [`tracing::Level`] for later use when setting up
/// tracing in the app module (see [`e62rs::app::logging`])
#[bearive::argdoc]
pub fn string_to_log_level(
    /// the string rep of the log level
    lvl: &str,
) -> tracing::Level {
    match lvl.to_lowercase().as_str() {
        "d" | "debug" | "dbg" => Level::DEBUG,
        "t" | "trace" | "trc" => Level::TRACE,
        "e" | "error" | "err" => Level::ERROR,
        "i" | "info" | "inf" => Level::INFO,
        "w" | "warn" | "wrn" => Level::WARN,
        _ => Level::ERROR,
    }
}

/// a repeatable function
pub trait Repeat {
    /// repeat n times
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    fn repeat(self, n: usize);
}

impl<F> Repeat for F
where
    F: Fn(),
{
    /// repeat n times
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    fn repeat(self, n: usize) {
        for _ in 0..n {
            self();
        }
    }
}

/// a repeatable function with collectable output
pub trait RepeatCollect {
    /// the type that'll be collected
    type Output;
    /// repeat n times and return the collected results
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    fn repeat_collect(self, n: usize) -> Vec<Self::Output>;
}

impl<F, R> RepeatCollect for F
where
    F: Fn() -> R,
{
    type Output = R;

    /// repeat n times and return the collected results
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    fn repeat_collect(self, n: usize) -> Vec<R> {
        (0..n).map(|_| self()).collect()
    }
}

/// a repeatable function with arguments
pub trait RepeatWith<Args> {
    /// the type to return
    type Output;

    /// repeat n times with args
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    /// * `args` - the arguments to pass to the repeated function
    fn repeat_with(self, n: usize, args: Args) -> Self::Output;
}

impl<F, A, R> RepeatWith<A> for F
where
    F: Fn(A) -> R,
    A: Clone,
{
    type Output = Vec<R>;

    /// repeat n times with args
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    /// * `args` - the arguments to pass to the repeated function
    fn repeat_with(self, n: usize, args: A) -> Vec<R> {
        (0..n).map(|_| self(args.clone())).collect()
    }
}

/// iterator repetition utils
pub trait IteratorRepeatExt: Iterator {
    /// repeat n times, returning the collected outputs
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    fn repeat_next(&mut self, n: usize) -> Vec<Self::Item>;

    /// repeat n times, ignoring the results
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    fn skip_n(&mut self, n: usize);
}

impl<I: Iterator> IteratorRepeatExt for I {
    /// repeat n times, returning the collected outputs
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    fn repeat_next(&mut self, n: usize) -> Vec<Self::Item> {
        (0..n).filter_map(|_| self.next()).collect()
    }

    /// repeat n times, ignoring the results
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    fn skip_n(&mut self, n: usize) {
        for _ in 0..n {
            self.next();
        }
    }
}

/// make a repeatable function
#[bearive::argdoc]
pub fn repeatable<F, R>(
    /// the function to make repeatable
    f: F,
) -> RepeatableOp<F>
where
    F: FnMut() -> R,
{
    RepeatableOp { f }
}

/// a repeatable operation
pub struct RepeatableOp<F> {
    /// the repeatable item
    pub f: F,
}

impl<F, R> RepeatableOp<F>
where
    F: FnMut() -> R,
{
    /// repeat n times
    #[bearive::argdoc]
    pub fn repeat(
        mut self,
        /// the number of repetitions
        n: usize,
    ) {
        for _ in 0..n {
            (self.f)();
        }
    }

    /// repeat n times, collecting the output
    #[bearive::argdoc]
    pub fn repeat_collect(
        mut self,
        /// the number of repetitions
        n: usize,
    ) -> Vec<R> {
        (0..n).map(|_| (self.f)()).collect()
    }

    /// repeat n times and return the last result
    #[bearive::argdoc]
    #[panic = "panics if `n` is less than 0"]
    pub fn repeat_last(
        mut self,
        /// the number of repetitions
        n: usize,
    ) -> R {
        assert!(n > 0, "repeat_last requires n > 0");
        let mut result = (self.f)();
        for _ in 1..n {
            result = (self.f)();
        }
        result
    }
}

/// a file writer
#[derive(Debug)]
pub enum FileWriter {
    /// write json data to a file
    Json {
        /// the path to write to
        path: PathBuf,
        /// the writer
        writer: BufWriter<File>,
        /// whether to format the json data
        pretty: bool,
    },

    /// write toml data to a file
    Toml {
        /// the path to write to
        path: PathBuf,
        /// the writer
        writer: BufWriter<File>,
        /// whether to format the toml data
        pretty: bool,
    },

    /// write plain text to a file
    Text {
        /// the path to write to
        path: PathBuf,
        /// the writer
        writer: BufWriter<File>,
    },

    #[cfg(target_os = "windows")]
    /// write to an ntfs ads (windows only)
    AltDataStream {
        /// the path to write to
        path: PathBuf,
        /// the writer
        writer: BufWriter<File>,
        /// use toml instead of json
        toml: bool,
    },
}

impl FileWriter {
    /// make a new json writer
    ///
    /// creates a new json file at the given path and then initializes a new `FileWriter` that,  
    /// when run, will write the given data passed to [`FileWriter::write`] to that file. pretty
    /// formatting is optional and based on the `pretty` parameter
    ///
    /// See [`FileWriter::write`]
    #[bearive::argdoc]
    #[error = "returns an error if it fails to create the json file"]
    pub fn json<P: AsRef<Path>>(
        /// the path to the file to access
        path: P,
        /// whether the resulting json should be formatted nicely
        pretty: bool,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .wrap_err_with(|| format!("failed to make json file: {}", path.display()))?;

        Ok(Self::Json {
            path,
            writer: BufWriter::new(file),
            pretty,
        })
    }

    /// make a new toml writer
    ///
    /// creates a new toml file at the given path and then initializes a new `FileWriter` that,  
    /// when run, will write the given data passed to [`FileWriter::write`] to that file. pretty
    /// formatting is optional and based on the `pretty` parameter
    ///
    /// See [`FileWriter::write`]
    #[bearive::argdoc]
    #[error = "returns an error if it fails to create the toml file"]
    pub fn toml<P: AsRef<Path>>(
        /// the path to the file to access
        path: P,
        /// whether the resulting toml should be formatted nicely
        pretty: bool,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .wrap_err_with(|| format!("failed to make toml file: {}", path.display()))?;

        Ok(Self::Toml {
            path,
            writer: BufWriter::new(file),
            pretty,
        })
    }

    /// make a new text writer
    ///
    /// creates a new text file at the given path and then initializes a new [`FileWriter`] that,
    /// when run, will write the given data passed to [`FileWriter::write`] to that file.
    ///
    /// See [`FileWriter::write`]
    #[bearive::argdoc]
    pub fn text<P: AsRef<Path>>(
        /// the path to the file to access
        path: P,
    ) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .wrap_err_with(|| format!("failed to make text file: {}", path.display()))?;

        Ok(Self::Text {
            path,
            writer: BufWriter::new(file),
        })
    }

    /// make a new ads writer
    ///
    /// opens a new alternate data stream on a path with a given name and then initializes a new
    /// [`FileWriter`] that, when run, will write the data passed to [`FileWriter::write`] to the
    /// ads. uses JSON when saving unless specified otherwise
    #[bearive::argdoc]
    pub fn ads<P: AsRef<Path>, S: AsRef<str>>(
        /// the path to make an ads on
        base_path: P,
        /// the name of the stream to open
        stream_name: S,
        /// use toml instead of json when serializing
        toml: bool,
    ) -> Result<Self> {
        let base = base_path.as_ref();
        let stream = stream_name.as_ref();
        let ads_path = format!("{}:{}", base.display(), stream);
        let path = PathBuf::from(&ads_path);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .wrap_err_with(|| {
                format!(
                    "failed to make ads '{}' on file '{}'",
                    stream,
                    base.display()
                )
            })?;

        Ok(Self::AltDataStream {
            path,
            writer: BufWriter::new(file),
            toml,
        })
    }

    /// write serializable data to the file
    ///
    /// takes a serializable value and, depending on the format specified when making the current
    /// [`FileWriter`], serializes said value into either TOML or JSON, and then writes it into the
    /// path specified on initialization.
    #[bearive::argdoc]
    pub fn write<T: Serialize>(
        &mut self,
        /// the data to write to the file/stream (must implement `Serialize`)
        data: &T,
    ) -> Result<()> {
        match self {
            Self::Json {
                path,
                writer,
                pretty,
            } => {
                let serialized = if *pretty {
                    serde_json::to_vec_pretty(data)
                } else {
                    serde_json::to_vec(data)
                }
                .wrap_err_with(|| {
                    format!("failed to serialize data to json for {}", path.display())
                })?;

                writer
                    .write_all(&serialized)
                    .wrap_err_with(|| format!("failed to write json to {}", path.display()))?;
            }

            Self::Toml {
                path,
                writer,
                pretty,
            } => {
                let serialized = if *pretty {
                    toml::to_string(data)
                } else {
                    toml::to_string_pretty(data)
                }
                .wrap_err_with(|| {
                    format!("failed to serialize data to toml for {}", path.display())
                })?;

                writer
                    .write_all(serialized.as_bytes())
                    .wrap_err_with(|| format!("failed to write toml to {}", path.display()))?;
            }

            Self::Text { path, writer } => {
                let serialized = serde_json::to_string(data).wrap_err_with(|| {
                    format!("failed to serialize data for text file {}", path.display())
                })?;

                writer
                    .write_all(serialized.as_bytes())
                    .wrap_err_with(|| format!("failed to write to text file {}", path.display()))?;
            }

            #[cfg(target_os = "windows")]
            Self::AltDataStream { path, writer, toml } => {
                let serialized = if *toml {
                    toml::to_string(data).wrap_err_with(|| {
                        format!("failed to serialize data for ads {}", path.display())
                    })?
                } else {
                    serde_json::to_string(data).wrap_err_with(|| {
                        format!("failed to serialize data for ads {}", path.display())
                    })?
                };

                writer
                    .write_all(serialized.as_bytes())
                    .wrap_err_with(|| format!("failed to write to ads {}", path.display()))?;
            }
        }

        Ok(())
    }

    /// write raw text to the file
    ///
    /// takes any value that can be used as a string and saves it to the path specified when making
    /// the current [`FileWriter`] instance
    #[bearive::argdoc]
    pub fn write_text<S: AsRef<str>>(
        &mut self,
        /// the string to write to the initialized file/stream
        text: S,
    ) -> Result<()> {
        let bytes = text.as_ref().as_bytes();

        match self {
            Self::Json { path, writer, .. }
            | Self::Toml { path, writer, .. }
            | Self::Text { path, writer, .. } => {
                writer
                    .write_all(bytes)
                    .wrap_err_with(|| format!("failed to write text to {}", path.display()))?;
            }
            #[cfg(target_os = "windows")]
            Self::AltDataStream { path, writer, .. } => {
                writer
                    .write_all(bytes)
                    .wrap_err_with(|| format!("failed to write text to ads {}", path.display()))?;
            }
        }

        Ok(())
    }

    /// flush the internal buffer to disk
    ///
    /// see [`Write::flush`]
    pub fn flush(&mut self) -> Result<()> {
        match self {
            Self::Json { path, writer, .. }
            | Self::Toml { path, writer, .. }
            | Self::Text { path, writer, .. } => {
                writer
                    .flush()
                    .wrap_err_with(|| format!("failed to flush buff to {}", path.display()))?;
            }
            #[cfg(target_os = "windows")]
            Self::AltDataStream { path, writer, .. } => {
                writer
                    .flush()
                    .wrap_err_with(|| format!("failed to flush buff to {}", path.display()))?;
            }
        }

        Ok(())
    }

    /// get the path associated with this writer
    ///
    /// simply returns the path specified when initializing the current [`FileWriter`]
    pub fn path(&self) -> &Path {
        match self {
            Self::Json { path, .. } | Self::Toml { path, .. } | Self::Text { path, .. } => path,
            #[cfg(target_os = "windows")]
            Self::AltDataStream { path, .. } => path,
        }
    }
}

impl Drop for FileWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// a guard that runs a closure when dropped
///
/// used by the [`crate::defer`] macro
pub struct DeferGuard<F: FnOnce()> {
    /// the closure to run
    pub func: Option<F>,
}

impl<F: FnOnce()> Drop for DeferGuard<F> {
    fn drop(&mut self) {
        if let Some(func) = self.func.take() {
            func();
        }
    }
}
