//! utilities used across e62rs
use {
    crate::getopt,
    base64::{Engine, engine::general_purpose},
    color_eyre::eyre::Result,
    reqwest::header::{AUTHORIZATION, HeaderMap},
    serde::Deserialize,
    std::{
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
    },
    tracing::Level,
};

/// deserialize a string into a boolean
///
/// # Arguments
///
/// * `deserializer` - the deserializer
///
/// # Errors
///
/// returns an error if it fails to deserialize the deserializer
pub fn deserialize_bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s == "t")
}

/// deserialize a string into a list of post ids
///
/// # Arguments
///
/// * `deserializer` - the deserializer
///
/// # Errors
///
/// returns an error if it fails to deserialize the deserializer
pub fn deserialize_post_ids<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
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

/// make an auth header based on the loaded config
///
/// # Errors
///
/// returns an error if it fails to convert `auth_value` to a [`reqwest::header::HeaderValue`]
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
/// # Arguments
///
/// * `path` - the path to shorten
/// * `max_len` - the max length for a single segment
pub fn shorten_path(path: &str, max_len: usize) -> String {
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
/// # Arguments
///
/// * `file_path` - the path to the file to make an ads on
/// * `stream_name` - the name of the ads stream
/// * `data` - the data to put into the ads stream
///
/// # Errors
///
/// returns an error if it fails to open `ads_path`  
/// returns an error if it fails to write `data` to `ads_path`
pub fn write_to_ads<P: AsRef<Path>>(file_path: P, stream_name: &str, data: &str) -> Result<usize> {
    let file_path = file_path.as_ref();
    let ads_path = format!("{}:{}", file_path.display(), stream_name);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&ads_path)?;

    file.write_all(data.as_bytes())?;
    Ok(data.len())
}

/// check if there's internet access
pub fn check_for_internet() -> bool {
    reqwest::blocking::get(crate::getopt!(http.api_url)).is_ok()
}

/// write some json data to a given file
///
/// # Arguments
///
/// * `file_path` - the path to the json file
/// * `data` - the data to write to `file_path`
///
/// # Errors
///
/// returns an error if it fails to open `file_path`  
/// returns an error if it fails to write `data` to `file_path`
pub fn _write_to_json<P: AsRef<Path>>(file_path: P, data: String) -> Result<()> {
    let file_path = file_path.as_ref();
    let json_path = format!("{}.json", file_path.display());

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&json_path)?;

    file.write_all(data.as_bytes())?;
    Ok(())
}

/// convert a string to a log level
///
/// # Arguments
///
/// * `lvl` - the string rep of the log level
pub fn string_to_log_level(lvl: &str) -> tracing::Level {
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
    fn repeat_next(&mut self, n: usize) -> Vec<Self::Item> {
        (0..n).filter_map(|_| self.next()).collect()
    }

    fn skip_n(&mut self, n: usize) {
        for _ in 0..n {
            self.next();
        }
    }
}

/// make a repeatable function
pub fn repeatable<F, R>(f: F) -> RepeatableOp<F>
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
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    pub fn repeat(mut self, n: usize) {
        for _ in 0..n {
            (self.f)();
        }
    }

    /// repeat n times, collecting the output
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    pub fn repeat_collect(mut self, n: usize) -> Vec<R> {
        (0..n).map(|_| (self.f)()).collect()
    }

    /// repeat n times and return the last result
    ///
    /// # Arguments
    ///
    /// * `n` - the number of times to repeat
    ///
    /// # Panics
    ///
    /// panics if `n` is less than 0
    pub fn repeat_last(mut self, n: usize) -> R {
        assert!(n > 0, "repeat_last requires n > 0");
        let mut result = (self.f)();
        for _ in 1..n {
            result = (self.f)();
        }
        result
    }
}
