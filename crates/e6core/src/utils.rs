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
