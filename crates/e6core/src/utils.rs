use {
    base64::{Engine, engine::general_purpose},
    e6cfg::LoginCfg,
    reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue},
    serde::Deserialize,
    std::{
        fs::OpenOptions,
        io::Write,
        path::{Path, PathBuf},
    },
};

/// converts a string into a boolean
pub fn deserialize_bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s == "t")
}

/// converts a string into a list of post ids
///
/// e.g.
/// `{1283,1828,1822}` would become `vec![1283, 1828, 1822]`
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

/// converts the current configuration for login into an auth header
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

#[cfg(test)]
mod tests {
    use {
        super::*,
        serde::de::{
            IntoDeserializer,
            value::{Error as DeError, StrDeserializer},
        },
        std::fs,
        tempfile::tempdir,
    };

    #[test]
    fn deserialize_bool_from_str_returns_true_for_t() {
        let deserializer: StrDeserializer<DeError> = "t".into_deserializer();
        let result = deserialize_bool_from_str(deserializer).unwrap();
        assert!(result);
    }

    #[test]
    fn deserialize_bool_from_str_returns_false_for_f() {
        let deserializer: StrDeserializer<DeError> = "f".into_deserializer();
        let result = deserialize_bool_from_str(deserializer).unwrap();
        assert!(!result);
    }

    #[test]
    fn deserialize_bool_from_str_returns_false_for_empty_string() {
        let deserializer: StrDeserializer<DeError> = "".into_deserializer();
        let result = deserialize_bool_from_str(deserializer).unwrap();
        assert!(!result);
    }

    #[test]
    fn deserialize_bool_from_str_returns_false_for_arbitrary_string() {
        let deserializer: StrDeserializer<DeError> = "anything_else".into_deserializer();
        let result = deserialize_bool_from_str(deserializer).unwrap();
        assert!(!result);
    }

    #[test]
    fn deserialize_post_ids_parses_single_id() {
        let deserializer: StrDeserializer<DeError> = "{1283}".into_deserializer();
        let result = deserialize_post_ids(deserializer).unwrap();
        assert_eq!(result, vec![1283]);
    }

    #[test]
    fn deserialize_post_ids_parses_multiple_ids() {
        let deserializer: StrDeserializer<DeError> = "{1283,1828,1822}".into_deserializer();
        let result = deserialize_post_ids(deserializer).unwrap();
        assert_eq!(result, vec![1283, 1828, 1822]);
    }

    #[test]
    fn deserialize_post_ids_handles_empty_braces() {
        let deserializer: StrDeserializer<DeError> = "{}".into_deserializer();
        let result = deserialize_post_ids(deserializer).unwrap();
        assert_eq!(result, Vec::<i64>::new());
    }

    #[test]
    fn deserialize_post_ids_handles_whitespace() {
        let deserializer: StrDeserializer<DeError> = "{  1283  ,  1828  }".into_deserializer();
        let result = deserialize_post_ids(deserializer).unwrap();
        assert_eq!(result, vec![1283, 1828]);
    }

    #[test]
    fn deserialize_post_ids_returns_empty_vec_for_string_without_braces() {
        let deserializer: StrDeserializer<DeError> = "1283,1828".into_deserializer();
        let result = deserialize_post_ids(deserializer).unwrap();
        assert_eq!(result, Vec::<i64>::new());
    }

    #[test]
    fn deserialize_post_ids_returns_error_for_invalid_number() {
        let deserializer: StrDeserializer<DeError> = "{1283,invalid,1822}".into_deserializer();
        let result = deserialize_post_ids(deserializer);
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_post_ids_handles_negative_numbers() {
        let deserializer: StrDeserializer<DeError> = "{-1283,1828}".into_deserializer();
        let result = deserialize_post_ids(deserializer).unwrap();
        assert_eq!(result, vec![-1283, 1828]);
    }

    #[test]
    fn create_auth_header_creates_valid_header() {
        let login_cfg = LoginCfg {
            username: "testuser".to_string(),
            api_key: "testkey".to_string(),
        };

        let headers = create_auth_header(&login_cfg).unwrap();
        let auth_value = headers.get(AUTHORIZATION).unwrap();

        let expected_auth_str = "testuser:testkey";
        let expected_encoded = general_purpose::STANDARD.encode(expected_auth_str);
        let expected_value = format!("Basic {}", expected_encoded);

        assert_eq!(auth_value.to_str().unwrap(), expected_value);
    }

    #[test]
    fn create_auth_header_handles_special_characters() {
        let login_cfg = LoginCfg {
            username: "user@example.com".to_string(),
            api_key: "key!@#$%^&*()".to_string(),
        };

        let headers = create_auth_header(&login_cfg).unwrap();
        assert!(headers.get(AUTHORIZATION).is_some());
    }

    #[test]
    fn create_auth_header_handles_empty_credentials() {
        let login_cfg = LoginCfg {
            username: "".to_string(),
            api_key: "".to_string(),
        };

        let headers = create_auth_header(&login_cfg).unwrap();
        assert!(headers.get(AUTHORIZATION).is_some());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn write_to_ads_creates_alternate_data_stream() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_file.txt");

        fs::write(&file_path, "main content")?;

        let stream_name = "test_stream";
        let data = "stream data content";
        let bytes_written = write_to_ads(&file_path, stream_name, data)?;

        assert_eq!(bytes_written, data.len());

        let ads_path = format!("{}:{}", file_path.display(), stream_name);
        let read_data = fs::read_to_string(&ads_path)?;
        assert_eq!(read_data, data);

        Ok(())
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn write_to_ads_overwrites_existing_stream() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_file.txt");

        fs::write(&file_path, "main content")?;

        let stream_name = "test_stream";
        write_to_ads(&file_path, stream_name, "first data")?;
        let bytes_written = write_to_ads(&file_path, stream_name, "second data")?;

        assert_eq!(bytes_written, "second data".len());

        let ads_path = format!("{}:{}", file_path.display(), stream_name);
        let read_data = fs::read_to_string(&ads_path)?;
        assert_eq!(read_data, "second data");

        Ok(())
    }

    #[test]
    fn write_to_json_creates_json_file() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_file.txt");

        let json_data = r#"{"key":"value","number":42}"#.to_string();
        write_to_json(&file_path, json_data.clone())?;

        let json_path = format!("{}.json", file_path.display());
        let read_data = fs::read_to_string(&json_path)?;
        assert_eq!(read_data, json_data);

        Ok(())
    }

    #[test]
    fn write_to_json_overwrites_existing_file() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_file.txt");

        let first_json = r#"{"first":"data"}"#.to_string();
        let second_json = r#"{"second":"data"}"#.to_string();

        write_to_json(&file_path, first_json)?;
        write_to_json(&file_path, second_json.clone())?;

        let json_path = format!("{}.json", file_path.display());
        let read_data = fs::read_to_string(&json_path)?;
        assert_eq!(read_data, second_json);

        Ok(())
    }

    #[test]
    fn write_to_json_handles_empty_string() -> anyhow::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_file.txt");

        write_to_json(&file_path, String::new())?;

        let json_path = format!("{}.json", file_path.display());
        let read_data = fs::read_to_string(&json_path)?;
        assert_eq!(read_data, "");

        Ok(())
    }

    #[test]
    fn shorten_path_returns_unchanged_when_components_under_limit() {
        let path = "short/path/name";
        let result = shorten_path(path, 10);
        assert_eq!(result, "short/path/name");
    }

    #[test]
    fn shorten_path_truncates_long_components() {
        let path = "verylongcomponent/another/path";
        let result = shorten_path(path, 5);
        assert_eq!(result, "veryl/anoth/path");
    }

    #[test]
    fn shorten_path_handles_single_component() {
        let path = "verylongfilename.txt";
        let result = shorten_path(path, 8);
        assert_eq!(result, "verylong");
    }

    #[test]
    fn shorten_path_converts_backslashes_to_forward_slashes() {
        let path = r"windows\style\path";
        let result = shorten_path(path, 20);
        assert_eq!(result, "windows/style/path");
    }

    #[test]
    fn shorten_path_handles_mixed_separators() {
        let path = r"mixed/and\various\separators";
        let result = shorten_path(path, 20);
        assert_eq!(result, "mixed/and/various/separators");
    }

    #[test]
    fn shorten_path_handles_empty_path() {
        let path = "";
        let result = shorten_path(path, 5);
        assert_eq!(result, "");
    }

    #[test]
    fn shorten_path_handles_zero_max_len() {
        let path = "some/path";
        let result = shorten_path(path, 0);
        assert_eq!(result, "");
    }

    #[test]
    fn shorten_path_preserves_unicode_correctly() {
        let path = "unicode/path/文件名";
        let result = shorten_path(path, 10);
        assert!(result.starts_with("unicode/path/"));
    }

    #[test]
    fn shorten_path_handles_absolute_paths() {
        let path = "/absolute/path/to/file";
        let result = shorten_path(path, 4);
        assert!(result.starts_with("/"));
    }

    #[test]
    fn shorten_path_handles_current_and_parent_dirs() {
        let path = "./relative/../path";
        let result = shorten_path(path, 10);
        assert_eq!(result, "./relative/../path");
    }
}
