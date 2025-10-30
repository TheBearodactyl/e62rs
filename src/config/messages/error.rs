#[derive(Debug, thiserror::Error)]
pub enum LogConfigError {
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Missing required variable '{variable}' in template '{template}'")]
    MissingVariable { template: String, variable: String },

    #[error("Invalid template syntax in '{template}': {reason}")]
    InvalidTemplate { template: String, reason: String },

    #[error("Configuration validation failed: {0}")]
    ValidationError(String),
}
