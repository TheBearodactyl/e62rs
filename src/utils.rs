use {
    crate::config::options::LoginCfg,
    base64::{Engine, engine::general_purpose},
    color_eyre::eyre::Result,
    reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue},
    serde::Deserialize,
    std::{
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
    },
    tracing::Level,
};

pub fn deserialize_bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s == "t")
}

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

pub fn create_auth_header(login_cfg: &LoginCfg) -> Result<HeaderMap> {
    let auth_str = format!(
        "{}:{}",
        login_cfg.clone().username,
        login_cfg.clone().api_key,
    );
    let encoded = general_purpose::STANDARD.encode(&auth_str);
    let auth_value = format!("Basic {}", encoded);
    let mut headers = HeaderMap::new();

    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

    Ok(headers)
}

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

#[macro_export]
macro_rules! fmt_value {
    () => {
        |v| format!("{}", v)
    };
    (debug) => {
        |v| format!("{:?}", v)
    };
}
