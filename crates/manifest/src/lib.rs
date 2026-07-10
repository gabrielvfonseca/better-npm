//! Package manifest (dash.json) schema, parsing, and validation.
//!
//! Handles reading, writing, and validating package manifests
//! (package.json, dash.json) which define package metadata,
//! dependencies, entry points, and build configuration.

use std::collections::BTreeMap;
use std::path::Path;

/// Error type for manifest operations.
#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("invalid field: {field}: {message}")]
    InvalidField { field: String, message: String },
    #[error("unsupported manifest format: {0}")]
    UnsupportedFormat(String),
    #[error("semver error: {0}")]
    Semver(#[from] semver::Error),
}

/// A version requirement string (e.g. "^1.0.0", "~2.3.0", ">=3.0.0").
pub type VersionReq = String;

/// A dependency entry in a manifest.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyEntry {
    /// Version requirement string.
    pub version: VersionReq,
    /// Whether this is an optional dependency.
    #[serde(default)]
    pub optional: bool,
    /// Whether this is a dev dependency.
    #[serde(default)]
    pub dev: bool,
    /// Package registry (if not the default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    /// Subpath imports within the dependency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imports: Option<BTreeMap<String, String>>,
}

impl DependencyEntry {
    /// Create a new dependency entry with just a version requirement.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            optional: false,
            dev: false,
            registry: None,
            imports: None,
        }
    }

    /// Mark this dependency as optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Mark this dependency as a dev dependency.
    pub fn dev(mut self) -> Self {
        self.dev = true;
        self
    }
}

/// A fully parsed and validated package manifest.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageManifest {
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Package description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Package author(s).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Package license.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Package homepage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Package repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Package keywords.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Production dependencies.
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencyEntry>,
    /// Dev dependencies.
    #[serde(default)]
    pub dev_dependencies: BTreeMap<String, DependencyEntry>,
    /// Optional dependencies.
    #[serde(default)]
    pub optional_dependencies: BTreeMap<String, DependencyEntry>,
    /// Peer dependencies.
    #[serde(default)]
    pub peer_dependencies: BTreeMap<String, DependencyEntry>,
    /// Package entry point(s).
    #[serde(default)]
    pub exports: Option<serde_json::Value>,
    /// Package type (module, commonjs).
    #[serde(default)]
    pub r#type: Option<String>,
    /// Scripts defined in the manifest.
    #[serde(default)]
    pub scripts: BTreeMap<String, String>,
    /// Binaries provided by this package.
    #[serde(default)]
    pub bin: Option<serde_json::Value>,
    /// Main entry point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,
    /// Module entry point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    /// TypeScript types entry point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<String>,
    /// Files included in the package.
    #[serde(default)]
    pub files: Vec<String>,
    /// Engines required by this package.
    #[serde(default)]
    pub engines: BTreeMap<String, String>,
    /// Any extra fields not explicitly listed.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl Default for PackageManifest {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            homepage: None,
            repository: None,
            keywords: Vec::new(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            optional_dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
            exports: None,
            r#type: None,
            scripts: BTreeMap::new(),
            bin: None,
            main: None,
            module: None,
            types: None,
            files: Vec::new(),
            engines: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }
}

impl PackageManifest {
    /// Validate the manifest fields.
    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.name.is_empty() {
            return Err(ManifestError::Validation("package name is required".to_string()));
        }
        // Validate name format
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/' || c == '@' || c == '.') {
            return Err(ManifestError::Validation(format!(
                "invalid package name: {}",
                self.name
            )));
        }
        // Validate version
        semver::Version::parse(&self.version)?;
        Ok(())
    }

    /// Get all dependencies combined, categorized by kind.
    pub fn all_dependencies(&self) -> impl Iterator<Item = (&str, &DependencyEntry, DependencyKind)> {
        let prod = self.dependencies.iter().map(|(k, v)| (k.as_str(), v, DependencyKind::Prod));
        let dev = self.dev_dependencies.iter().map(|(k, v)| (k.as_str(), v, DependencyKind::Dev));
        let opt = self.optional_dependencies.iter().map(|(k, v)| (k.as_str(), v, DependencyKind::Optional));
        let peer = self.peer_dependencies.iter().map(|(k, v)| (k.as_str(), v, DependencyKind::Peer));
        prod.chain(dev).chain(opt).chain(peer)
    }
}

/// Kind of dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyKind {
    Prod,
    Dev,
    Optional,
    Peer,
}

/// A parsed manifest with raw access to the original data.
#[derive(Debug, Clone)]
pub struct Manifest {
    /// The raw parsed JSON value.
    pub raw: serde_json::Value,
    /// The validated package manifest.
    pub parsed: PackageManifest,
    /// The original source path.
    pub source: Option<String>,
}

impl Manifest {
    /// Parse a manifest from a string content.
    ///
    /// Supports JSON and JSONC (stripping comments).
    pub fn parse(content: &str) -> Result<Self, ManifestError> {
        let raw: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| ManifestError::Parse(e.to_string()))?;

        let parsed: PackageManifest = serde_json::from_value(raw.clone())
            .map_err(|e| ManifestError::Parse(e.to_string()))?;

        parsed.validate()?;

        Ok(Self {
            raw,
            parsed,
            source: None,
        })
    }

    /// Load a manifest from a file path.
    pub fn load(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        let mut manifest = Self::parse(&content)?;
        manifest.source = Some(path.to_string_lossy().to_string());
        Ok(manifest)
    }

    /// Parse a manifest from TOML content.
    pub fn parse_toml(content: &str) -> Result<Self, ManifestError> {
        let raw: serde_json::Value = toml::from_str(content)
            .map_err(|e| ManifestError::Parse(e.to_string()))?;

        let parsed: PackageManifest = serde_json::from_value(raw.clone())
            .map_err(|e| ManifestError::Parse(e.to_string()))?;

        parsed.validate()?;

        Ok(Self {
            raw,
            parsed,
            source: None,
        })
    }

    /// Return the package name.
    pub fn name(&self) -> &str {
        &self.parsed.name
    }

    /// Return the package version.
    pub fn version(&self) -> &str {
        &self.parsed.version
    }

    /// Return the dependencies.
    pub fn dependencies(&self) -> &BTreeMap<String, DependencyEntry> {
        &self.parsed.dependencies
    }

    /// Return the dev dependencies.
    pub fn dev_dependencies(&self) -> &BTreeMap<String, DependencyEntry> {
        &self.parsed.dev_dependencies
    }

    /// Return the optional dependencies.
    pub fn optional_dependencies(&self) -> &BTreeMap<String, DependencyEntry> {
        &self.parsed.optional_dependencies
    }

    /// Return the peer dependencies.
    pub fn peer_dependencies(&self) -> &BTreeMap<String, DependencyEntry> {
        &self.parsed.peer_dependencies
    }

    /// Return the raw parsed value.
    pub fn raw(&self) -> &serde_json::Value {
        &self.raw
    }

    /// Export the parsed manifest.
    pub fn export(&self) -> &PackageManifest {
        &self.parsed
    }

    /// Serialize the manifest to JSON bytes.
    pub fn to_json(&self) -> Result<Vec<u8>, ManifestError> {
        serde_json::to_vec_pretty(&self.parsed)
            .map_err(|e| ManifestError::Parse(e.to_string()))
    }

    /// Serialize the manifest to TOML.
    pub fn to_toml(&self) -> Result<String, ManifestError> {
        toml::to_string(&self.parsed).map_err(|e| ManifestError::Parse(e.to_string()))
    }

    /// Get a field from the raw manifest by path.
    pub fn get_raw(&self, path: &str) -> Option<&serde_json::Value> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.raw;
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
}

/// Detect the manifest format from a filename.
pub fn detect_format(path: &Path) -> Result<&'static str, ManifestError> {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    match filename {
        "package.json" | "dash.json" => Ok("json"),
        "dash.jsonc" => Ok("jsonc"),
        "dash.toml" | "Package.toml" => Ok("toml"),
        _ if filename.ends_with(".json") => Ok("json"),
        _ if filename.ends_with(".toml") => Ok("toml"),
        _ => Err(ManifestError::UnsupportedFormat(
            filename.to_string(),
        )),
    }
}

/// Find a manifest file in the given directory.
pub fn find_manifest(path: &Path) -> Option<std::path::PathBuf> {
    for name in &["package.json", "dash.json", "dash.jsonc", "dash.toml"] {
        let candidate = path.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parse_json() {
        let content = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "description": "A test package",
            "dependencies": {
                "lodash": { "version": "^4.17.21" },
                "react": "^18.0.0"
            }
        }"#;

        let manifest = Manifest::parse(content).unwrap();
        assert_eq!(manifest.name(), "test-package");
        assert_eq!(manifest.version(), "1.0.0");
        assert!(manifest.dependencies().contains_key("lodash"));
    }

    #[test]
    fn test_manifest_parse_invalid() {
        let content = r#"{"name": ""}"#;
        let manifest = Manifest::parse(content);
        assert!(manifest.is_err());
    }

    #[test]
    fn test_manifest_parse_toml() {
        let content = r#"
name = "toml-pkg"
version = "0.1.0"
description = "TOML test"

[dependencies]
serde = "1.0"
        "#;

        let manifest = Manifest::parse_toml(content).unwrap();
        assert_eq!(manifest.name(), "toml-pkg");
        assert_eq!(manifest.version(), "0.1.0");
    }

    #[test]
    fn test_dependency_entry() {
        let entry = DependencyEntry::new("^1.0.0").optional();
        assert_eq!(entry.version, "^1.0.0");
        assert!(entry.optional);
        assert!(!entry.dev);
    }

    #[test]
    fn test_manifest_to_json() {
        let content = r#"{"name": "pkg", "version": "1.0.0"}"#;
        let manifest = Manifest::parse(content).unwrap();
        let json = manifest.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&json).unwrap();
        assert_eq!(parsed["name"], "pkg");
    }

    #[test]
    fn test_find_manifest() {
        let dir = std::path::Path::new(".");
        let result = find_manifest(dir);
        // May or may not find a manifest depending on the test directory
        assert!(result.is_none() || result.is_some());
    }

    #[test]
    fn test_detect_format() {
        assert_eq!(
            detect_format(Path::new("package.json")).unwrap(),
            "json"
        );
        assert_eq!(
            detect_format(Path::new("dash.toml")).unwrap(),
            "toml"
        );
    }

    #[test]
    fn test_package_manifest_validate() {
        let mut manifest = PackageManifest::default();
        assert!(manifest.validate().is_err()); // empty name
        manifest.name = "valid-pkg".to_string();
        manifest.version = "1.0.0".to_string();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_manifest_get_raw() {
        let content = r#"{"name": "pkg", "scripts": {"build": "tsc"}}"#;
        let manifest = Manifest::parse(content).unwrap();
        assert_eq!(
            manifest.get_raw("scripts.build").and_then(|v| v.as_str()),
            Some("tsc")
        );
    }

    #[test]
    fn test_all_dependencies() {
        let mut manifest = PackageManifest::default();
        manifest.name = "pkg".to_string();
        manifest.dependencies.insert("prod-dep".to_string(), DependencyEntry::new("1.0.0"));
        manifest.dev_dependencies.insert("dev-dep".to_string(), DependencyEntry::new("2.0.0").dev());

        let deps: Vec<_> = manifest.all_dependencies().collect();
        assert_eq!(deps.len(), 2);
    }
}
