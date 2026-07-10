//! Configuration management: layered profiles, env overrides, CLI args.
//!
//! Dash loads configuration from multiple sources in order of increasing
//! priority: defaults → config files → environment variables → CLI args.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Error type for configuration operations.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("key not found: {0}")]
    KeyNotFound(String),
    #[error("deserialize error: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("invalid config: {0}")]
    Invalid(String),
}

/// Priority level for a configuration source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigPriority {
    Defaults = 0,
    File = 1,
    Env = 2,
    Cli = 3,
}

/// Source of a configuration value.
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// A configuration file.
    File(PathBuf),
    /// An environment variable.
    Env(String, String),
    /// Default value baked into the binary.
    Default,
    /// A command-line argument.
    Cli(String),
}

impl ConfigSource {
    /// Return the priority level.
    pub fn priority(&self) -> ConfigPriority {
        match self {
            ConfigSource::Default => ConfigPriority::Defaults,
            ConfigSource::File(_) => ConfigPriority::File,
            ConfigSource::Env(_, _) => ConfigPriority::Env,
            ConfigSource::Cli(_) => ConfigPriority::Cli,
        }
    }
}

/// Layered configuration with tracking of value provenance.
#[derive(Debug, Clone)]
pub struct Config {
    inner: serde_json::Value,
    sources: BTreeMap<String, ConfigSource>,
}

impl Config {
    /// Create an empty configuration.
    pub fn empty() -> Self {
        Self {
            inner: serde_json::Value::Object(Default::default()),
            sources: BTreeMap::new(),
        }
    }

    /// Load configuration from a list of file paths. Files are loaded in order;
    /// later files override earlier ones.
    pub fn load(files: &[PathBuf]) -> Result<Self, ConfigError> {
        let mut config = Self::empty();

        for file in files {
            if file.exists() {
                let content = std::fs::read_to_string(file)?;
                let value = parse_config_content(&content, file)
                    .map_err(|e| ConfigError::Parse(format!("in {}: {}", file.display(), e)))?;
                config.set_source(value, ConfigSource::File(file.clone()));
            } else {
                // Skip missing files silently, except for known config paths
            }
        }

        Ok(config)
    }

    /// Get a value at a dot-separated key path, deserializing to the requested type.
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T, ConfigError> {
        let value = self.resolve_key(key).ok_or_else(|| ConfigError::KeyNotFound(key.to_string()))?;
        let cloned = value.clone();
        serde_json::from_value(cloned).map_err(ConfigError::Deserialize)
    }

    /// Get a value or return a default.
    pub fn get_or<T: serde::de::DeserializeOwned>(&self, key: &str, default: T) -> T {
        self.get(key).unwrap_or(default)
    }

    /// Set a value at a dot-separated key path.
    pub fn set(&mut self, key: &str, value: impl serde::Serialize) {
        let val = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
        self.set_at_key(key, val);
        self.sources.insert(key.to_string(), ConfigSource::Default);
    }

    /// Set a value with a specific source.
    pub fn set_with_source(&mut self, key: &str, value: impl serde::Serialize, source: ConfigSource) {
        let val = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
        self.set_at_key(key, val);
        self.sources.insert(key.to_string(), source);
    }

    /// Merge another config into this one. Keys from `other` override keys in `self`.
    pub fn merge(&mut self, other: Config) {
        self.merge_value(other.inner);
        for (key, source) in other.sources {
            if source.priority() >= self.source_priority(&key) {
                self.sources.insert(key, source);
            }
        }
    }

    /// Check if a key exists.
    pub fn has(&self, key: &str) -> bool {
        self.resolve_key(key).is_some()
    }

    /// Return the source for a given key, if tracked.
    pub fn source(&self, key: &str) -> Option<&ConfigSource> {
        self.sources.get(key)
    }

    /// Return all keys in the config.
    pub fn keys(&self) -> Vec<String> {
        self.sources.keys().cloned().collect()
    }

    /// Return the raw JSON value.
    pub fn raw(&self) -> &serde_json::Value {
        &self.inner
    }

    fn set_source(&mut self, value: serde_json::Value, source: ConfigSource) {
        let priority = source.priority();
        self.merge_value(value);
        // Track sources for top-level keys
        if let serde_json::Value::Object(map) = &self.inner {
            for key in map.keys() {
                let current_priority = self.source_priority(key);
                if priority >= current_priority {
                    self.sources.insert(key.clone(), source.clone());
                }
            }
        }
    }

    fn resolve_key(&self, key: &str) -> Option<&serde_json::Value> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &self.inner;
        for part in parts {
            match current {
                serde_json::Value::Object(map) => {
                    current = map.get(part)?;
                }
                _ => return None,
            }
        }
        Some(current)
    }

    fn set_at_key(&mut self, key: &str, value: serde_json::Value) {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            if let Some(map) = self.inner.as_object_mut() {
                map.insert(parts[0].to_string(), value);
            }
            return;
        }

        // Navigate/create nested structure
        let mut current = &mut self.inner;
        for &part in &parts[..parts.len() - 1] {
            if let Some(map) = current.as_object_mut() {
                if !map.contains_key(part) {
                    map.insert(part.to_string(), serde_json::Value::Object(Default::default()));
                }
                current = map.get_mut(part).unwrap();
            } else {
                // Can't set nested key on non-object
                return;
            }
        }

        if let Some(map) = current.as_object_mut() {
            map.insert(parts[parts.len() - 1].to_string(), value);
        }
    }

    fn merge_value(&mut self, value: serde_json::Value) {
        merge_json(&mut self.inner, value);
    }

    fn source_priority(&self, key: &str) -> ConfigPriority {
        self.sources
            .get(key)
            .map(|s| s.priority())
            .unwrap_or(ConfigPriority::Defaults)
    }
}

fn merge_json(left: &mut serde_json::Value, right: serde_json::Value) {
    match (left, right) {
        (left @ &mut serde_json::Value::Object(_), serde_json::Value::Object(right_map)) => {
            let left_map = left.as_object_mut().unwrap();
            for (k, v) in right_map {
                if let Some(existing) = left_map.get_mut(&k) {
                    merge_json(existing, v);
                } else {
                    left_map.insert(k, v);
                }
            }
        }
        (left, right) => *left = right,
    }
}

fn parse_config_content(content: &str, path: &Path) -> Result<serde_json::Value, ConfigError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "json" => Ok(serde_json::from_str(content)?),
        "yaml" | "yml" => {
            serde_yaml::from_str(content).map_err(|e| ConfigError::Parse(e.to_string()))
        }
        "toml" => {
            let value: serde_json::Value = toml::from_str(content)
                .map_err(|e| ConfigError::Parse(e.to_string()))?;
            Ok(value)
        }
        other => Err(ConfigError::Parse(format!("unsupported config format: {}", other))),
    }
}

/// Load the default configuration for Dash by searching standard locations.
pub fn load_config() -> Result<Config, ConfigError> {
    let mut config_files = Vec::new();

    // Standard config locations
    if let Some(config_dir) = dirs::config_dir() {
        let dash_dir = config_dir.join("dash");
        for filename in &["dash.json", "dash.yaml", "dash.yml", "dash.toml"] {
            config_files.push(dash_dir.join(filename));
        }
    }

    // Project-local config files
    for filename in &["dash.json", "dash.yaml", "dash.yml", "dash.toml", ".dash.json"] {
        config_files.push(PathBuf::from(filename));
    }

    let mut config = Config::load(&config_files)?;

    // Apply environment variable overrides (e.g. DASH_REGISTRY, DASH_CACHE_DIR)
    for (key, value) in std::env::vars() {
        if let Some(config_key) = key.strip_prefix("DASH_") {
            let config_key = config_key.to_lowercase().replace('_', ".");
            config.set_with_source(&config_key, value.clone(), ConfigSource::Env(key, value));
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_empty() {
        let config = Config::empty();
        assert!(!config.has("registry"));
    }

    #[test]
    fn test_config_set_and_get() {
        let mut config = Config::empty();
        config.set("registry.url", "https://registry.example.com");
        let url: String = config.get("registry.url").unwrap();
        assert_eq!(url, "https://registry.example.com");
    }

    #[test]
    fn test_config_get_or() {
        let config = Config::empty();
        let val: String = config.get_or("nonexistent", "default".to_string());
        assert_eq!(val, "default");
    }

    #[test]
    fn test_config_merge() {
        let mut base = Config::empty();
        base.set("a", 1);

        let mut other = Config::empty();
        other.set("b", 2);

        base.merge(other);
        assert_eq!(base.get::<i32>("a").unwrap(), 1);
        assert_eq!(base.get::<i32>("b").unwrap(), 2);
    }

    #[test]
    fn test_config_merge_overrides() {
        let mut base = Config::empty();
        base.set("key", "old");

        let mut other = Config::empty();
        other.set("key", "new");

        base.merge(other);
        assert_eq!(base.get::<String>("key").unwrap(), "new");
    }

    #[test]
    fn test_config_nested_keys() {
        let mut config = Config::empty();
        config.set("registry.auth.token", "my-token");
        let token: String = config.get("registry.auth.token").unwrap();
        assert_eq!(token, "my-token");
    }

    #[test]
    fn test_config_missing_key() {
        let config = Config::empty();
        let result: Result<String, _> = config.get("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_has() {
        let mut config = Config::empty();
        config.set("present", 42);
        assert!(config.has("present"));
        assert!(!config.has("absent"));
    }

    #[test]
    fn test_parse_config_json() {
        let path = Path::new("test.json");
        let content = r#"{"registry": {"url": "https://example.com"}}"#;
        let value = parse_config_content(content, path).unwrap();
        assert_eq!(
            value["registry"]["url"].as_str().unwrap(),
            "https://example.com"
        );
    }
}
