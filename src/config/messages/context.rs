use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    variables: HashMap<String, String>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.variables.insert(key.into(), value.into());
        self
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.variables.get(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.variables.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.variables.keys()
    }

    pub fn len(&self) -> usize {
        self.variables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
}

impl From<HashMap<String, String>> for TemplateContext {
    fn from(variables: HashMap<String, String>) -> Self {
        Self { variables }
    }
}

impl FromIterator<(String, String)> for TemplateContext {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self {
            variables: iter.into_iter().collect(),
        }
    }
}
