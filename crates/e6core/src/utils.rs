use std::{fs::OpenOptions, io::Write, path::Path};

use base64::{Engine, engine::general_purpose};
use e6cfg::LoginCfg;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::Deserialize;

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

pub fn create_auth_header(login_cfg: &LoginCfg) -> anyhow::Result<HeaderMap> {
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

pub fn write_to_ads<P: AsRef<Path>>(
    file_path: P,
    stream_name: &str,
    data: &str,
) -> anyhow::Result<usize> {
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

pub fn write_to_json<P: AsRef<Path>>(file_path: P, data: String) -> anyhow::Result<()> {
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
