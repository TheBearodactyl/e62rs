use {
    crate::config::messages::error::LogConfigError,
    color_eyre::eyre::{Result, eyre},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageTemplate {
    pub template: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub required_vars: Vec<String>,

    #[serde(default)]
    pub defaults: HashMap<String, String>,
}

impl MessageTemplate {
    pub fn new<S: Into<String>>(template: S) -> Self {
        Self {
            template: template.into(),
            description: None,
            required_vars: Vec::new(),
            defaults: HashMap::new(),
        }
    }

    pub fn with_description<S: Into<String>>(mut self, desc: S) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_required_vars(mut self, vars: Vec<String>) -> Self {
        self.required_vars = vars;
        self
    }

    pub fn with_defaults(mut self, defaults: HashMap<String, String>) -> Self {
        self.defaults = defaults;
        self
    }

    pub fn extract_vars(&self) -> Result<Vec<String>> {
        let mut variables = Vec::new();
        let mut chars = self.template.chars().peekable();
        let mut in_placeholder = false;
        let mut current_var = String::new();

        for ch in chars {
            match ch {
                '{' => {
                    if in_placeholder {
                        return Err(eyre!(LogConfigError::InvalidTemplate {
                            template: self.template.clone(),
                            reason: "Nested braces are not allowed".to_string()
                        }));
                    }

                    in_placeholder = true;
                    current_var.clear();
                }
                '}' => {
                    if !in_placeholder {
                        return Err(eyre!(LogConfigError::InvalidTemplate {
                            template: self.template.clone(),
                            reason: "Unmatched closing brace".to_string()
                        }));
                    }

                    if current_var.is_empty() {
                        return Err(eyre!(LogConfigError::InvalidTemplate {
                            template: self.template.clone(),
                            reason: "Empty placeholder".to_string()
                        }));
                    }

                    variables.push(current_var.clone());
                    in_placeholder = false;
                }
                _ => {
                    if in_placeholder {
                        current_var.push(ch);
                    }
                }
            }
        }

        if in_placeholder {
            return Err(eyre!(LogConfigError::InvalidTemplate {
                template: self.template.clone(),
                reason: "Unmatched opening brace".to_string()
            }));
        }

        Ok(variables)
    }

    pub fn validate(&self) -> Result<()> {
        self.extract_vars()?;
        Ok(())
    }
}
