use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KvLintConfig {
    pub ignored_rules: Vec<String>,
    pub strict_mode: bool,
    pub target_models: Vec<String>,
    pub cache_threshold_tokens: Option<usize>,
    pub ignore_paths: Vec<String>,
}

impl Default for KvLintConfig {
    fn default() -> Self {
        Self {
            ignored_rules: Vec::new(),
            strict_mode: false,
            target_models: vec!["generic".to_string()],
            cache_threshold_tokens: None,
            ignore_paths: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    ParseError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::ParseError(msg) => write!(f, "Failed to parse config: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Parses raw TOML string into `KvLintConfig`.
pub fn parse_config(raw_toml: &str) -> Result<KvLintConfig, ConfigError> {
    toml::from_str(raw_toml).map_err(|e| ConfigError::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let raw = r#"
            ignored_rules = ["KV001", "KV003"]
            strict_mode = true
            target_models = ["claude-3-5-sonnet", "gpt-4o"]
            cache_threshold_tokens = 1024
            ignore_paths = ["tests/*", "fixtures/*"]
        "#;

        let config = parse_config(raw).expect("should parse valid config");
        assert_eq!(config.ignored_rules, vec!["KV001", "KV003"]);
        assert!(config.strict_mode);
        assert_eq!(config.target_models, vec!["claude-3-5-sonnet", "gpt-4o"]);
        assert_eq!(config.cache_threshold_tokens, Some(1024));
        assert_eq!(config.ignore_paths, vec!["tests/*", "fixtures/*"]);
    }

    #[test]
    fn test_parse_empty_config_uses_defaults() {
        let raw = "";
        let config = parse_config(raw).expect("empty config should parse to defaults");
        assert_eq!(config, KvLintConfig::default());
        assert!(!config.strict_mode);
        assert_eq!(config.target_models, vec!["generic".to_string()]);
        assert!(config.ignored_rules.is_empty());
        assert!(config.ignore_paths.is_empty());
        assert_eq!(config.cache_threshold_tokens, None);
    }

    #[test]
    fn test_parse_partial_config() {
        let raw = r#"
            ignored_rules = ["KV002"]
            strict_mode = true
        "#;
        let config = parse_config(raw).expect("partial config should parse");
        assert_eq!(config.ignored_rules, vec!["KV002"]);
        assert!(config.strict_mode);
        assert_eq!(config.target_models, vec!["generic".to_string()]);
    }

    #[test]
    fn test_parse_invalid_toml_syntax() {
        let raw = "invalid = [toml, syntax";
        let err = parse_config(raw).expect_err("should fail on invalid syntax");
        match err {
            ConfigError::ParseError(msg) => {
                assert!(!msg.is_empty());
            }
        }
    }
}
