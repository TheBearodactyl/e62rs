use {
    crate::config::messages::{
        builder::MessagesConfigBuilder, context::TemplateContext, error::LogConfigError,
        template::MessageTemplate,
    },
    color_eyre::eyre::{Context, Result, eyre},
    config::{Config, File},
    serde::{Deserialize, Serialize},
    std::{collections::HashMap, path::Path},
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LogMessageConfig {
    pub templates: HashMap<String, MessageTemplate>,

    #[serde(default)]
    pub global_defaults: HashMap<String, String>,

    #[serde(default = "default_strict")]
    pub strict: bool,
}

fn default_strict() -> bool {
    true
}

impl LogMessageConfig {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            global_defaults: HashMap::new(),
            strict: true,
        }
    }

    pub fn load() -> Result<Self> {
        Self::from_file("messages.toml")
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let cfg = Config::builder()
            .add_source(File::from(path))
            .build()
            .wrap_err_with(|| format!("Failed to load messages from {}", path.display()))?;

        let log_cfg: Self = cfg
            .try_deserialize()
            .wrap_err("Failed to deserialize configuration")?;

        log_cfg.validate()?;

        Ok(log_cfg)
    }

    pub fn builder() -> MessagesConfigBuilder {
        MessagesConfigBuilder::new()
    }

    pub fn add_template<S: Into<String>>(
        &mut self,
        name: S,
        template: MessageTemplate,
    ) -> Result<()> {
        let name = name.into();
        template.validate()?;
        self.templates.insert(name, template);
        Ok(())
    }

    pub fn get_template(&self, name: &str) -> Option<&MessageTemplate> {
        self.templates.get(name)
    }

    pub fn format_message(&self, template_name: &str, context: &TemplateContext) -> Result<String> {
        let template = self
            .templates
            .get(template_name)
            .ok_or_else(|| eyre!(LogConfigError::TemplateNotFound(template_name.to_string())))?;

        self.format_with_template(template, template_name, context)
    }

    fn format_with_template(
        &self,
        template: &MessageTemplate,
        template_name: &str,
        context: &TemplateContext,
    ) -> Result<String> {
        let mut result = template.template.clone();
        let variables = template.extract_vars()?;

        for var in variables {
            let value = self.resolve_variable(&var, template, context)?;

            if self.strict && value.is_none() {
                return Err(eyre!(LogConfigError::MissingVariable {
                    template: template_name.to_string(),
                    variable: var.clone()
                }));
            }

            if let Some(value) = value {
                result = result.replace(&format!("{{{}}}", var), &value);
            }
        }

        Ok(result)
    }

    fn resolve_variable(
        &self,
        var: &str,
        template: &MessageTemplate,
        context: &TemplateContext,
    ) -> Result<Option<String>> {
        if let Some(value) = context.get(var) {
            return Ok(Some(value.clone()));
        }

        if let Some(value) = template.defaults.get(var) {
            return Ok(Some(value.clone()));
        }

        if let Some(value) = self.global_defaults.get(var) {
            return Ok(Some(value.clone()));
        }

        Ok(None)
    }

    pub fn validate(&self) -> Result<()> {
        for (name, template) in &self.templates {
            template
                .validate()
                .wrap_err_with(|| format!("Invalid template '{}'", name))?;
        }

        Ok(())
    }

    pub fn template_names(&self) -> impl Iterator<Item = &String> {
        self.templates.keys()
    }
}

impl Default for LogMessageConfig {
    fn default() -> Self {
        Self::new()
    }
}
