use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::dependency::{DependencyError, DependencySource};

#[derive(Debug, Deserialize, Serialize)]
pub struct MarkdownConfig {
    pub package: Package,
    #[serde(default)]
    pub dependencies: HashMap<String, toml::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    ReadError(#[from] std::io::Error),
    #[error("Failed to parse TOML: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Dependency error: {0}")]
    DependencyError(#[from] DependencyError),
    #[error("Config file not found: {0}")]
    NotFound(String),
}

impl MarkdownConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(ConfigError::NotFound(path.display().to_string()));
        }

        let content = fs::read_to_string(path)?;
        let config: MarkdownConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn parse_dependencies(&self) -> Result<HashMap<String, DependencySource>, ConfigError> {
        let mut deps = HashMap::new();

        for (name, value) in &self.dependencies {
            let source = DependencySource::parse(value)?;
            deps.insert(name.clone(), source);
        }

        Ok(deps)
    }

    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn default_template(name: &str) -> Self {
        Self {
            package: Package {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: Some("A collection of Markdown documentation files".to_string()),
            },
            dependencies: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[package]
name = "test-project"
version = "0.1.0"

[dependencies]
example = "github:owner/repo/README.md"
"#;

        let config: MarkdownConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.package.name, "test-project");
        assert_eq!(config.dependencies.len(), 1);
    }
}
