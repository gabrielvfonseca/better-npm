//! Dash manifest (dash.toml) — types, parsing, serialization, and validation
//! for the canonical Dash package manifest format.
//!
//! Supports:
//! - TOML parsing/serialization (primary format)
//! - JSON parsing/serialization (compat)
//! - Canonical CBOR encoding (for DASHCAP1 capsules)
//! - Validation per dash.toml.json schema
//! - Package.json import for M5 compat

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use toml;

use crate::{DependencyKind, ManifestError};

// ---------------------------------------------------------------------------
// Top-level Manifest
// ---------------------------------------------------------------------------

/// The Dash manifest root. Every field is optional — an absent JSON section
/// simply produces `None` (or defaults to empty for maps/vecs).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<PackageIdentity>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exports: Option<Exports>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<DependencyMap>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dev_dependencies: Option<DependencyMap>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer_dependencies: Option<DependencyMap>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional_dependencies: Option<DependencyMap>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Capabilities>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildDeclaration>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai: Option<AiBlock>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<Workspace>,
}

impl Manifest {
    /// Parse a `dash.json` string.
    pub fn from_json_str(input: &str) -> Result<Self, ManifestError> {
        let manifest: Manifest =
            serde_json::from_str(input).map_err(|e| ManifestError::Parse(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Load and parse a `dash.json` file from disk.
    pub fn from_json_path(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    /// Serialize to a JSON string.
    pub fn to_json_string(&self) -> Result<String, ManifestError> {
        serde_json::to_string_pretty(self).map_err(|e| ManifestError::Parse(e.to_string()))
    }

    /// Parse a `dash.toml` string.
    pub fn from_toml_str(input: &str) -> Result<Self, ManifestError> {
        let manifest: Manifest =
            toml::from_str(input).map_err(|e| ManifestError::Parse(e.to_string()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Load and parse a `dash.toml` file from disk.
    pub fn from_toml_path(path: &Path) -> Result<Self, ManifestError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_toml_str(&content)
    }

    /// Serialize to a TOML string with sorted keys (deterministic).
    pub fn to_toml_string(&self) -> Result<String, ManifestError> {
        toml::to_string_pretty(self).map_err(|e| ManifestError::Parse(e.to_string()))
    }

    /// Encode to canonical CBOR (self-described CBOR, tag 258).
    ///
    /// Encoding rules per CONTENT-ADDRESSING.md §3.1:
    /// - Map keys sorted lexicographically by bytewise comparison
    /// - Integers in smallest form
    /// - No floats
    /// - No indefinite-length encoding
    /// - Wrapped in tag 258 (self-described CBOR)
    pub fn to_canonical_cbor(&self) -> Result<Vec<u8>, ManifestError> {
        let mut inner_buf = Vec::new();
        ciborium::ser::into_writer(self, &mut inner_buf)
            .map_err(|e| ManifestError::Parse(format!("CBOR serialize: {}", e)))?;

        let inner: ciborium::Value = ciborium::de::from_reader(&inner_buf[..])
            .map_err(|e| ManifestError::Parse(format!("CBOR parse: {}", e)))?;

        let tagged = ciborium::Value::Tag(258, Box::new(inner));

        let mut buf = Vec::new();
        ciborium::ser::into_writer(&tagged, &mut buf)
            .map_err(|e| ManifestError::Parse(format!("CBOR wrap: {}", e)))?;

        Ok(buf)
    }

    /// Decode from canonical CBOR (expects tag 258, tolerates missing tag).
    pub fn from_canonical_cbor(data: &[u8]) -> Result<Self, ManifestError> {
        let val: ciborium::Value = ciborium::de::from_reader(data)
            .map_err(|e| ManifestError::Parse(format!("CBOR deserialize: {}", e)))?;

        let inner = match val {
            ciborium::Value::Tag(258, inner) => *inner,
            other => other,
        };

        let mut buf = Vec::new();
        ciborium::ser::into_writer(&inner, &mut buf)
            .map_err(|e| ManifestError::Parse(format!("CBOR re-serialize: {}", e)))?;

        let manifest: Self = ciborium::de::from_reader(&buf[..])
            .map_err(|e| ManifestError::Parse(format!("CBOR deserialize manifest: {}", e)))?;

        Ok(manifest)
    }

    /// Get the dependency map for a given dependency kind.
    pub fn deps(&self, kind: DependencyKind) -> &DependencyMap {
        static EMPTY: std::sync::LazyLock<DependencyMap> = std::sync::LazyLock::new(BTreeMap::new);
        match kind {
            DependencyKind::Prod => self.dependencies.as_ref().unwrap_or(&EMPTY),
            DependencyKind::Dev => self.dev_dependencies.as_ref().unwrap_or(&EMPTY),
            DependencyKind::Peer => self.peer_dependencies.as_ref().unwrap_or(&EMPTY),
            DependencyKind::Optional => &EMPTY,
        }
    }

    /// Validate all fields against the dash.toml schema rules.
    pub fn validate(&self) -> Result<(), ManifestError> {
        let pkg = self
            .package
            .as_ref()
            .ok_or_else(|| ManifestError::MissingField("package".to_string()))?;

        if pkg.name.is_empty() {
            return Err(ManifestError::Validation("package.name is required".to_string()));
        }
        if !is_valid_package_name(&pkg.name) {
            return Err(ManifestError::InvalidField {
                field: "package.name".to_string(),
                message: format!("invalid package name: {}", pkg.name),
            });
        }

        if let Err(e) = semver::Version::parse(&pkg.version) {
            return Err(ManifestError::InvalidField {
                field: "package.version".to_string(),
                message: format!("invalid semver version: {}", e),
            });
        }

        if let Some(ref license) = pkg.license {
            if !is_valid_spdx_expression(license) {
                return Err(ManifestError::InvalidField {
                    field: "package.license".to_string(),
                    message: format!("invalid SPDX license expression: {}", license),
                });
            }
        }

        if let Some(ref runtime) = pkg.runtime {
            for entry in runtime {
                if !is_valid_runtime_entry(entry) {
                    return Err(ManifestError::InvalidField {
                        field: "package.runtime".to_string(),
                        message: format!("invalid runtime entry: {}", entry),
                    });
                }
            }
        }

        if let Some(ref exports) = self.exports {
            if let Err(e) = validate_exports(exports) {
                return Err(ManifestError::InvalidField {
                    field: "exports".to_string(),
                    message: e,
                });
            }
        }

        if let Some(ref caps) = self.capabilities {
            validate_capabilities(caps)?;
        }

        if let Some(ref ai) = self.ai {
            ai.validate()?;
        }

        if let Some(ref ws) = self.workspace {
            if ws.members.is_empty() {
                return Err(ManifestError::Validation(
                    "workspace.members must not be empty".to_string(),
                ));
            }
        }

        if let Some(ref build) = self.build {
            if build.tool.is_empty() {
                return Err(ManifestError::Validation("build.tool is required".to_string()));
            }
        }

        Ok(())
    }
}

fn is_valid_package_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let rest = if let Some(stripped) = name.strip_prefix('@') {
        let slash_pos = stripped.find('/').map(|p| p + 1);
        match slash_pos {
            Some(end) => {
                let scope = &stripped[..end.saturating_sub(1)];
                if scope.is_empty() {
                    return false;
                }
                if !scope.chars().all(|c| {
                    c.is_ascii_lowercase()
                        || c.is_ascii_digit()
                        || c == '-'
                        || c == '.'
                        || c == '_'
                        || c == '~'
                }) {
                    return false;
                }
                &stripped[end..]
            },
            None => return false,
        }
    } else {
        name
    };
    if rest.is_empty() {
        return false;
    }
    rest.chars().all(|c| {
        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.' || c == '_' || c == '~'
    })
}

/// Validate SPDX license expression.
/// Uses a basic regex pattern since a full SPDX list is not bundled.
/// Pattern matches: SPDX license ID (alphanum, ., -, +) optionally combined with AND/OR/WITH.
fn is_valid_spdx_expression(license: &str) -> bool {
    if license.is_empty() {
        return false;
    }
    // Basic SPDX expression pattern: identifiers with optional AND/OR/WITH operators
    // SPDX license IDs: alphanumeric, dots, hyphens, plus signs
    // Examples: "MIT", "Apache-2.0", "GPL-2.0-or-later", "MIT AND Apache-2.0", "GPL-2.0-only WITH Bison-exception-2.2"
    let pattern = regex::Regex::new(
        r"^([A-Za-z0-9.\-+]+(?:[-+][A-Za-z0-9.\-+]+)*)(?:(?: WITH )?[A-Za-z0-9.\-+]+)*(?: (?:AND|OR) [A-Za-z0-9.\-+]+(?:[-+][A-Za-z0-9.\-+]+)*(?: WITH [A-Za-z0-9.\-+]+)*)*$"
    ).unwrap();
    pattern.is_match(license)
}

/// Validate a runtime entry like "node>=20" or "deno>=2".
fn is_valid_runtime_entry(entry: &str) -> bool {
    if entry.is_empty() {
        return false;
    }
    // Runtime name + optional semver range
    let pattern = regex::Regex::new(r"^[a-z][a-z0-9-]*(?:[><=~^!]=?[0-9][0-9a-z.\-]*)?$").unwrap();
    pattern.is_match(entry)
}

/// Validate exports map: keys must start with '.' or be '.', values must have 'types' + 'import'/'require'.
fn validate_exports(exports: &Exports) -> Result<(), String> {
    for (key, conditions) in exports {
        if key != "." && !key.starts_with('.') {
            return Err(format!("export key must start with '.' or be '.': {}", key));
        }
        if conditions.types.is_none() {
            return Err(format!("export '{}' must have 'types' field", key));
        }
        if conditions.import.is_none() && conditions.require.is_none() {
            return Err(format!("export '{}' must have 'import' or 'require' field", key));
        }
    }
    Ok(())
}

/// Validate capabilities: keys must be in allowed set, values must be string array or false.
fn validate_capabilities(caps: &Capabilities) -> Result<(), ManifestError> {
    let allowed_keys = ["net", "fs-read", "fs-write", "env", "subprocess"];
    let cap_values = [
        ("net", &caps.net),
        ("fs-read", &caps.fs_read),
        ("fs-write", &caps.fs_write),
        ("env", &caps.env),
    ];

    for (key, value) in cap_values {
        if !allowed_keys.contains(&key) {
            return Err(ManifestError::InvalidField {
                field: format!("capabilities.{}", key),
                message: format!(
                    "invalid capability key '{}', must be one of: {}",
                    key,
                    allowed_keys.join(", ")
                ),
            });
        }
        if let Some(CapabilityValue::Bool(true)) = value {
            return Err(ManifestError::InvalidField {
                field: format!("capabilities.{}", key),
                message: "capability must be false or an allowlist".to_string(),
            });
        }
        if let Some(CapabilityValue::List(list)) = value {
            if list.is_empty() {
                return Err(ManifestError::InvalidField {
                    field: format!("capabilities.{}", key),
                    message: "capability allowlist must not be empty".to_string(),
                });
            }
        }
    }

    // subprocess is Option<bool>, validate separately
    if let Some(true) = caps.subprocess {
        return Err(ManifestError::InvalidField {
            field: "capabilities.subprocess".to_string(),
            message: "capability must be false or omitted".to_string(),
        });
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Package Identity
// ---------------------------------------------------------------------------

/// Core package metadata. Every `dash.json` must have this section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackageIdentity {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<Vec<String>>,
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub module_type: Option<ModuleType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
}

/// Module system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleType {
    #[serde(rename = "module")]
    Module,
    #[serde(rename = "commonjs")]
    CommonJs,
}

// ---------------------------------------------------------------------------
// Exports
// ---------------------------------------------------------------------------

/// Typed, condition-based entry points.
pub type Exports = BTreeMap<String, ExportConditions>;

/// Condition map for an export subpath.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConditions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub types: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bun: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deno: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workerd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub development: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub production: Option<String>,
}

// ---------------------------------------------------------------------------
// Dependencies
// ---------------------------------------------------------------------------

/// A map of package name → dependency spec.
pub type DependencyMap = BTreeMap<String, DependencySpec>;

/// A dependency specification — either a simple version range string or an
/// extended object with grants, optionals, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencySpec {
    Simple(String),
    Extended {
        version: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        grants: Option<Vec<String>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        optional: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        singleton: Option<bool>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        registry: Option<String>,
    },
}

impl DependencySpec {
    pub fn version(&self) -> &str {
        match self {
            DependencySpec::Simple(v) => v,
            DependencySpec::Extended { version, .. } => version,
        }
    }

    pub fn grants(&self) -> Option<&[String]> {
        match self {
            DependencySpec::Simple(_) => None,
            DependencySpec::Extended { grants, .. } => grants.as_deref(),
        }
    }

    pub fn is_optional(&self) -> bool {
        match self {
            DependencySpec::Simple(_) => false,
            DependencySpec::Extended { optional, .. } => optional.unwrap_or(false),
        }
    }

    pub fn is_singleton(&self) -> bool {
        match self {
            DependencySpec::Simple(_) => false,
            DependencySpec::Extended { singleton, .. } => singleton.unwrap_or(false),
        }
    }

    pub fn registry(&self) -> Option<&str> {
        match self {
            DependencySpec::Simple(_) => None,
            DependencySpec::Extended { registry, .. } => registry.as_deref(),
        }
    }
}

// ---------------------------------------------------------------------------
// Capabilities
// ---------------------------------------------------------------------------

/// Runtime capabilities a package declares.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Capabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<CapabilityValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fs_read: Option<CapabilityValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fs_write: Option<CapabilityValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<CapabilityValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subprocess: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildCapabilities>,
}

/// A capability value: either `false` (denied) or an allowlist of strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CapabilityValue {
    Bool(bool),
    List(Vec<String>),
}

/// Build-time capability sandbox settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripts: Option<bool>,
}

// ---------------------------------------------------------------------------
// Build Declaration
// ---------------------------------------------------------------------------

/// Declarative, sandboxed build step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDeclaration {
    pub tool: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outputs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

// ---------------------------------------------------------------------------
// AI Block
// ---------------------------------------------------------------------------

/// Structured, machine-readable metadata for agent-based package selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiBlock {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(rename = "capabilities-declared", default, skip_serializing_if = "Option::is_none")]
    pub capabilities_declared: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stability: Option<StabilityLevel>,
    #[serde(rename = "side-effects", default, skip_serializing_if = "Option::is_none")]
    pub side_effects: Option<SideEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
    #[serde(rename = "tool-interfaces", default, skip_serializing_if = "Option::is_none")]
    pub tool_interfaces: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub examples: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    #[serde(rename = "deprecated-in-favor-of", default, skip_serializing_if = "Option::is_none")]
    pub deprecated_in_favor_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl AiBlock {
    fn validate(&self) -> Result<(), ManifestError> {
        if let Some(ref intent) = self.intent {
            if intent.trim().is_empty() {
                return Err(ManifestError::InvalidField {
                    field: "ai.intent".to_string(),
                    message: "intent must be non-empty".to_string(),
                });
            }
        }
        if let Some(ref se) = self.side_effects {
            match se {
                SideEffect::None
                | SideEffect::Filesystem
                | SideEffect::Network
                | SideEffect::Global => {},
            }
        }
        if let Some(ref stability) = self.stability {
            matches!(
                stability,
                StabilityLevel::Experimental
                    | StabilityLevel::Stable
                    | StabilityLevel::Maintenance
                    | StabilityLevel::Deprecated
            );
        }
        Ok(())
    }
}

/// Stability level for the AI block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StabilityLevel {
    #[serde(rename = "experimental")]
    Experimental,
    #[serde(rename = "stable")]
    Stable,
    #[serde(rename = "maintenance")]
    Maintenance,
    #[serde(rename = "deprecated")]
    Deprecated,
}

/// Side-effect classification for the AI block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SideEffect {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "filesystem")]
    Filesystem,
    #[serde(rename = "network")]
    Network,
    #[serde(rename = "global")]
    Global,
}

// ---------------------------------------------------------------------------
// Workspace
// ---------------------------------------------------------------------------

/// Workspace root configuration for monorepos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub members: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared: Option<BTreeMap<String, serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// package.json import
// ---------------------------------------------------------------------------

impl Manifest {
    /// Import a package.json value into a dash.json Manifest.
    ///
    /// This enables M5 compat — `dash init` reads package.json
    /// and produces the equivalent `dash.json`.
    pub fn from_package_json(value: serde_json::Value) -> Result<Self, ManifestError> {
        let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let version = value.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0").to_string();

        let package = PackageIdentity {
            name,
            version,
            description: value.get("description").and_then(|v| v.as_str()).map(String::from),
            license: value.get("license").and_then(|v| v.as_str()).map(String::from),
            authors: value
                .get("author")
                .and_then(|v| v.as_str())
                .map(|a| vec![a.to_string()])
                .or_else(|| {
                    value.get("contributors").and_then(|v| v.as_array()).map(|arr| {
                        arr.iter().filter_map(|v| v.as_str().map(String::from)).collect()
                    })
                }),
            repository: value.get("repository").and_then(|v| {
                v.as_str().map(String::from).or_else(|| {
                    v.as_object()
                        .and_then(|o| o.get("url"))
                        .and_then(|u| u.as_str().map(String::from))
                })
            }),
            runtime: None,
            module_type: value.get("type").and_then(|v| v.as_str()).and_then(|t| match t {
                "module" => Some(ModuleType::Module),
                "commonjs" => Some(ModuleType::CommonJs),
                _ => None,
            }),
            private: value.get("private").and_then(|v| v.as_bool()),
        };

        let dependencies = parse_dep_map(value.get("dependencies"));
        let dev_dependencies = parse_dep_map(value.get("devDependencies"));
        let peer_dependencies = parse_dep_map(value.get("peerDependencies"));

        let exports = value
            .get("exports")
            .and_then(|v| serde_json::from_value::<Exports>(v.clone()).ok());

        Ok(Manifest {
            package: Some(package),
            exports,
            dependencies,
            dev_dependencies,
            peer_dependencies,
            optional_dependencies: None,
            capabilities: None,
            build: None,
            ai: None,
            workspace: None,
        })
    }
}

fn parse_dep_map(value: Option<&serde_json::Value>) -> Option<DependencyMap> {
    let obj = value?.as_object()?;
    let mut map = BTreeMap::new();
    for (name, val) in obj {
        let spec = if let Some(version_str) = val.as_str() {
            DependencySpec::Simple(version_str.to_string())
        } else if let Some(obj) = val.as_object() {
            let version = obj.get("version").and_then(|v| v.as_str()).unwrap_or("*").to_string();
            let grants = obj
                .get("grants")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());
            let optional = obj.get("optional").and_then(|v| v.as_bool());
            let singleton = obj.get("singleton").and_then(|v| v.as_bool());
            let registry = obj.get("registry").and_then(|v| v.as_str()).map(String::from);
            DependencySpec::Extended { version, grants, optional, singleton, registry }
        } else {
            continue;
        };
        map.insert(name.clone(), spec);
    }
    Some(map)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_JSON: &str = r#"{
        "package": { "name": "test-pkg", "version": "1.0.0" }
    }"#;

    const VALID_JSON: &str = r#"{
        "package": {
            "name": "@acme/http-client",
            "version": "2.3.0",
            "description": "Typed, retrying HTTP client",
            "license": "Apache-2.0",
            "authors": ["Acme Eng <eng@acme.dev>"],
            "repository": "https://github.com/acme/http-client",
            "runtime": ["node>=20"],
            "type": "module",
            "private": false
        },
        "exports": {
            ".": { "types": "./dist/index.d.ts", "import": "./dist/index.js" },
            "./mock": { "types": "./dist/mock.d.ts", "import": "./dist/mock.js" }
        },
        "dependencies": {
            "@std/encoding": "^1.0",
            "zod": "^3.23",
            "native-tls": { "version": "^0.4", "grants": ["net:tls"] }
        },
        "dev_dependencies": { "vitest": "^2" },
        "peer_dependencies": { "typescript": { "version": ">=5.4", "optional": true } },
        "capabilities": {
            "net": ["https://api.acme.dev"],
            "fs-read": ["./config"],
            "fs-write": false,
            "env": ["ACME_API_KEY"],
            "subprocess": false,
            "build": { "network": false, "scripts": false }
        },
        "build": { "tool": "dash:tsc", "inputs": ["src/**"], "outputs": ["dist/**"], "target": "node" },
        "ai": {
            "summary": "HTTP client with automatic retries",
            "intent": "networking/http-client",
            "capabilities-declared": ["http.request", "http.retry"],
            "stability": "stable",
            "side-effects": "none",
            "docs": "./docs/agent.md",
            "tool-interfaces": "./ai/tools.json",
            "examples": "./ai/examples/",
            "supersedes": "@acme/legacy-http@^1"
        },
        "workspace": { "members": ["packages/*", "apps/*"] }
    }"#;

    #[test]
    fn test_parse_valid() {
        let manifest = Manifest::from_json_str(VALID_JSON).unwrap();
        let pkg = manifest.package.as_ref().unwrap();
        assert_eq!(pkg.name, "@acme/http-client");
        assert_eq!(pkg.version, "2.3.0");
    }

    #[test]
    fn test_json_roundtrip() {
        let manifest = Manifest::from_json_str(VALID_JSON).unwrap();
        let json_out = manifest.to_json_string().unwrap();
        let parsed_again = Manifest::from_json_str(&json_out).unwrap();
        assert_eq!(
            manifest.package.as_ref().map(|p| p.name.clone()),
            parsed_again.package.as_ref().map(|p| p.name.clone())
        );
    }

    #[test]
    fn test_minimal_manifest() {
        let manifest = Manifest::from_json_str(MINIMAL_JSON).unwrap();
        let pkg = manifest.package.as_ref().unwrap();
        assert_eq!(pkg.name, "test-pkg");
        assert_eq!(pkg.version, "1.0.0");
    }

    #[test]
    fn test_validation_missing_name() {
        let result = Manifest::from_json_str(r#"{"package": { "version": "1.0.0" }}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_invalid_version() {
        let result =
            Manifest::from_json_str(r#"{"package": { "name": "pkg", "version": "not-valid" }}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_missing_package() {
        let result = Manifest::from_json_str(r#"{"build": { "tool": "dash:tsc" }}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_cbor_roundtrip() {
        let manifest = Manifest::from_json_str(VALID_JSON).unwrap();
        let cbor = manifest.to_canonical_cbor().unwrap();
        let parsed = Manifest::from_canonical_cbor(&cbor).unwrap();
        assert_eq!(
            manifest.package.as_ref().map(|p| p.name.clone()),
            parsed.package.as_ref().map(|p| p.name.clone())
        );
    }

    #[test]
    fn test_cbor_has_tag_258() {
        let manifest = Manifest::from_json_str(MINIMAL_JSON).unwrap();
        let cbor = manifest.to_canonical_cbor().unwrap();
        assert!(cbor.len() >= 3);
        assert_eq!(cbor[0], 0xd9, "expected tag header");
        assert_eq!(cbor[1], 0x01, "expected tag 258 high byte");
        assert_eq!(cbor[2], 0x02, "expected tag 258 low byte");
    }

    #[test]
    fn test_full_manifest() {
        let manifest = Manifest::from_json_str(VALID_JSON).unwrap();
        let pkg = manifest.package.as_ref().unwrap();
        assert_eq!(pkg.module_type, Some(ModuleType::Module));
    }

    #[test]
    fn test_ai_block() {
        let manifest = Manifest::from_json_str(VALID_JSON).unwrap();
        let ai = manifest.ai.as_ref().unwrap();
        assert_eq!(ai.intent.as_deref(), Some("networking/http-client"));
    }

    #[test]
    fn test_capabilities_allowlists() {
        let manifest = Manifest::from_json_str(VALID_JSON).unwrap();
        let caps = manifest.capabilities.as_ref().unwrap();
        if let Some(CapabilityValue::List(urls)) = &caps.net {
            assert_eq!(urls.len(), 1);
        } else {
            panic!("net should be a List");
        }
    }

    #[test]
    fn test_build_declaration() {
        let manifest = Manifest::from_json_str(VALID_JSON).unwrap();
        let build = manifest.build.as_ref().unwrap();
        assert_eq!(build.tool, "dash:tsc");
    }

    #[test]
    fn test_workspace_config() {
        let manifest = Manifest::from_json_str(VALID_JSON).unwrap();
        let ws = manifest.workspace.as_ref().unwrap();
        assert_eq!(ws.members.len(), 2);
    }

    #[test]
    fn test_deps_method() {
        let manifest = Manifest::from_json_str(VALID_JSON).unwrap();
        let prod = manifest.deps(DependencyKind::Prod);
        assert!(prod.contains_key("zod"));
        let dev = manifest.deps(DependencyKind::Dev);
        assert!(dev.contains_key("vitest"));
    }

    #[test]
    fn test_from_package_json() {
        let pj: serde_json::Value = serde_json::json!({
            "name": "@acme/pkg",
            "version": "1.0.0",
            "type": "module",
            "dependencies": { "lodash": "^4.17.21" }
        });
        let manifest = Manifest::from_package_json(pj).unwrap();
        assert_eq!(manifest.package.as_ref().unwrap().name, "@acme/pkg");
        assert!(manifest.dependencies.as_ref().unwrap().contains_key("lodash"));
    }

    #[test]
    fn test_dependency_spec_accessors() {
        let simple = DependencySpec::Simple("^1.0".to_string());
        assert_eq!(simple.version(), "^1.0");
        assert!(simple.grants().is_none());
    }

    #[test]
    fn test_workspace_empty_members() {
        let result = Manifest::from_json_str(
            r#"{
            "package": { "name": "ws-root", "version": "1.0.0" },
            "workspace": { "members": [] }
        }"#,
        );
        assert!(result.is_err());
    }

    // TOML I/O tests
    const VALID_TOML: &str = r#"
package = { name = "@acme/http-client", version = "2.3.0", license = "Apache-2.0" }
exports = { "." = { types = "./dist/index.d.ts", import = "./dist/index.js" } }
dependencies = { "@std/encoding" = "^1.0", zod = "^3.23" }
capabilities = { net = ["https://api.acme.dev"], fs-read = ["./config"], fs-write = false, env = ["KEY"], subprocess = false }
build = { tool = "dash:tsc", inputs = ["src/**"], outputs = ["dist/**"] }
ai = { intent = "networking/http-client", stability = "stable", "side-effects" = "none" }
"#;

    #[test]
    fn test_toml_roundtrip() {
        let manifest = Manifest::from_toml_str(VALID_TOML).unwrap();
        let toml_out = manifest.to_toml_string().unwrap();
        let parsed_again = Manifest::from_toml_str(&toml_out).unwrap();
        assert_eq!(
            manifest.package.as_ref().map(|p| p.name.clone()),
            parsed_again.package.as_ref().map(|p| p.name.clone())
        );
        assert_eq!(
            manifest.package.as_ref().map(|p| p.version.clone()),
            parsed_again.package.as_ref().map(|p| p.version.clone())
        );
    }

    #[test]
    fn test_toml_parse_valid() {
        let manifest = Manifest::from_toml_str(VALID_TOML).unwrap();
        let pkg = manifest.package.as_ref().unwrap();
        assert_eq!(pkg.name, "@acme/http-client");
        assert_eq!(pkg.version, "2.3.0");
        assert_eq!(pkg.license.as_deref(), Some("Apache-2.0"));
    }

    #[test]
    fn test_toml_minimal() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
"#;
        let manifest = Manifest::from_toml_str(toml).unwrap();
        let pkg = manifest.package.as_ref().unwrap();
        assert_eq!(pkg.name, "test-pkg");
        assert_eq!(pkg.version, "1.0.0");
    }

    // Validation rule tests
    #[test]
    fn test_validation_invalid_package_name_uppercase() {
        let toml = r#"
package = { name = "InvalidName", version = "1.0.0" }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid package name"));
    }

    #[test]
    fn test_validation_invalid_package_name_underscore_only() {
        // underscore is allowed but must be lowercase/dash separated
        let toml = r#"
package = { name = "valid_name", version = "1.0.0" }
"#;
        let result = Manifest::from_toml_str(toml);
        // underscore is allowed in our validation
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_missing_license() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_ok()); // license is optional
    }

    #[test]
    fn test_validation_invalid_spdx_license() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0", license = "Not-A-Valid-License!" }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("invalid SPDX license expression"));
    }

    #[test]
    fn test_validation_valid_spdx_license() {
        let valid_licenses = [
            "MIT",
            "Apache-2.0",
            "GPL-3.0-only",
            "MIT OR Apache-2.0",
            "GPL-2.0-only WITH Bison-exception-2.2",
        ];
        for license in valid_licenses {
            let toml = format!(
                r#"
package = {{ name = "test-pkg", version = "1.0.0", license = "{}" }}
"#,
                license
            );
            let result = Manifest::from_toml_str(&toml);
            assert!(result.is_ok(), "license '{}' should be valid: {:?}", license, result.err());
        }
    }

    #[test]
    fn test_validation_invalid_runtime() {
        // Runtime must start with lowercase letter, then lowercase/digits/hyphens
        let invalid_runtimes = ["123invalid", "INVALID", "Node>=20", "node@20"];
        for rt in invalid_runtimes {
            let toml = format!(
                r#"
package = {{ name = "test-pkg", version = "1.0.0", runtime = ["{}"] }}
"#,
                rt
            );
            let result = Manifest::from_toml_str(&toml);
            assert!(result.is_err(), "runtime '{}' should be invalid", rt);
            let err = result.unwrap_err().to_string();
            assert!(err.contains("invalid runtime entry"), "error for '{}': {}", rt, err);
        }
    }

    #[test]
    fn test_validation_valid_runtime() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0", runtime = ["node>=20", "deno>=2", "bun>=1.2"] }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_exports_key_must_start_with_dot() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
exports = { "invalid" = { types = "./a.d.ts", import = "./a.js" } }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("export key must start with '.'"));
    }

    #[test]
    fn test_validation_exports_must_have_types() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
exports = { "." = { import = "./a.js" } }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("must have 'types' field"));
    }

    #[test]
    fn test_validation_exports_must_have_import_or_require() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
exports = { "." = { types = "./a.d.ts" } }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("must have 'import' or 'require' field"));
    }

    #[test]
    fn test_validation_valid_exports() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
exports = { "." = { types = "./a.d.ts", import = "./a.js" }, "./sub" = { types = "./b.d.ts", require = "./b.js" } }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_capabilities_invalid_key() {
        // Capabilities struct has fixed fields; serde rejects unknown fields at deserialization
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
capabilities = { "invalid-key" = ["something"] }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        // Error will be a deserialization error about unknown field
    }

    #[test]
    fn test_validation_capabilities_bool_true_invalid() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
capabilities = { net = true }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("capability must be false or an allowlist"));
    }

    #[test]
    fn test_validation_capabilities_empty_list_invalid() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
capabilities = { net = [] }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("capability allowlist must not be empty"));
    }

    #[test]
    fn test_validation_capabilities_subprocess_true_invalid() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
capabilities = { subprocess = true }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("capability must be false or omitted"));
    }

    #[test]
    fn test_validation_ai_intent_empty_invalid() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
ai = { intent = "" }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("intent must be non-empty"));
    }

    #[test]
    fn test_validation_ai_intent_valid() {
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
ai = { intent = "networking/http-client", stability = "stable", "side-effects" = "none" }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_ai_stability_invalid() {
        // Invalid enum value rejected at deserialization
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
ai = { intent = "test", stability = "invalid", "side-effects" = "none" }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_ai_side_effects_invalid() {
        // Invalid enum value rejected at deserialization
        let toml = r#"
package = { name = "test-pkg", version = "1.0.0" }
ai = { intent = "test", stability = "stable", "side-effects" = "invalid" }
"#;
        let result = Manifest::from_toml_str(toml);
        assert!(result.is_err());
    }
}
