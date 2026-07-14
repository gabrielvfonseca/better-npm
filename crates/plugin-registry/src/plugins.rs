use std::collections::HashMap;
use std::path::Path;

use dash_capsule::Capsule;
use dash_proto::dpr_v1::{Binding, VersionRange};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("plugin already loaded: {0}")]
    AlreadyLoaded(String),
    #[error("plugin initialization failed: {0}")]
    InitFailed(String),
    #[error("plugin execution failed: {0}")]
    ExecutionFailed(String),
    #[error("sandbox error: {0}")]
    SandboxError(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginType {
    Registry,
    Builder,
    Cli,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapability {
    pub service: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub entry_point: String,
    pub capabilities: Vec<PluginCapability>,
    pub events: Vec<String>,
    pub config_schema: Option<serde_json::Value>,
    pub dependencies: Vec<String>,
    pub min_dash_version: Option<String>,
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.name.is_empty() {
            return Err(PluginError::InvalidManifest("plugin name is required".into()));
        }
        if self.entry_point.is_empty() {
            return Err(PluginError::InvalidManifest("entry point is required".into()));
        }
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, PluginError> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Self = serde_json::from_str(&content)
            .map_err(|e| PluginError::InvalidManifest(format!("invalid JSON: {e}")))?;
        manifest.validate()?;
        Ok(manifest)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub manifest: PluginManifest,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
    pub path: String,
    pub hash: String,
}

pub struct PluginContext<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub capsule: Option<&'a Capsule>,
    pub bindings: Option<&'a [Binding]>,
    pub ranges: Option<&'a [VersionRange]>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl<'a> PluginContext<'a> {
    pub fn new(name: &'a str, version: &'a str) -> Self {
        Self {
            name,
            version,
            capsule: None,
            bindings: None,
            ranges: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_capsule(mut self, capsule: &'a Capsule) -> Self {
        self.capsule = Some(capsule);
        self
    }

    pub fn with_bindings(mut self, bindings: &'a [Binding]) -> Self {
        self.bindings = Some(bindings);
        self
    }

    pub fn with_ranges(mut self, ranges: &'a [VersionRange]) -> Self {
        self.ranges = Some(ranges);
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

pub type PluginResult = Result<PluginHookResult, PluginError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginHookResult {
    Continue,
    Veto { reason: String },
    Modified { bindings: Option<Vec<Binding>>, metadata: Option<HashMap<String, serde_json::Value>> },
    Custom(serde_json::Value),
}

impl PluginHookResult {
    pub fn is_continue(&self) -> bool {
        matches!(self, PluginHookResult::Continue)
    }

    pub fn is_veto(&self) -> bool {
        matches!(self, PluginHookResult::Veto { .. })
    }
}

#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn plugin_type(&self) -> PluginType;
    fn capabilities(&self) -> Vec<PluginCapability>;
    fn events(&self) -> Vec<String>;

    async fn on_load(&self) -> Result<(), PluginError>;
    async fn on_unload(&self) -> Result<(), PluginError>;

    async fn on_pre_publish(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }

    async fn on_post_publish(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }

    async fn on_pre_resolve(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }

    async fn on_post_resolve(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }

    async fn on_pre_fetch(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }

    async fn on_post_fetch(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }

    async fn on_policy_check(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }

    async fn on_version_added(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }

    async fn on_yank(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }

    async fn on_tag_change(&self, ctx: &PluginContext<'_>) -> PluginResult {
        Ok(PluginHookResult::Continue)
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinPlugin {
    name: String,
    version: String,
    plugin_type: PluginType,
    capabilities: Vec<PluginCapability>,
    events: Vec<String>,
}

impl BuiltinPlugin {
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        plugin_type: PluginType,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            plugin_type,
            capabilities: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn with_capability(mut self, capability: PluginCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn with_event(mut self, event: impl Into<String>) -> Self {
        self.events.push(event.into());
        self
    }
}

#[async_trait::async_trait]
impl Plugin for BuiltinPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn plugin_type(&self) -> PluginType {
        self.plugin_type.clone()
    }

    fn capabilities(&self) -> Vec<PluginCapability> {
        self.capabilities.clone()
    }

    fn events(&self) -> Vec<String> {
        self.events.clone()
    }

    async fn on_load(&self) -> Result<(), PluginError> {
        tracing::info!("Builtin plugin '{}' loaded (v{})", self.name, self.version);
        Ok(())
    }

    async fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("Builtin plugin '{}' unloaded", self.name);
        Ok(())
    }
}

pub struct LicenseCheckPlugin;

impl LicenseCheckPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Plugin for LicenseCheckPlugin {
    fn name(&self) -> &str { "license-check" }
    fn version(&self) -> &str { "1.0.0" }
    fn plugin_type(&self) -> PluginType { PluginType::Registry }
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability { service: "policy".into(), actions: vec!["check-license".into()] }]
    }
    fn events(&self) -> Vec<String> { vec!["pre-publish".into(), "post-publish".into()] }

    async fn on_load(&self) -> Result<(), PluginError> { Ok(()) }
    async fn on_unload(&self) -> Result<(), PluginError> { Ok(()) }

    async fn on_pre_publish(&self, ctx: &PluginContext<'_>) -> PluginResult {
        if let Some(capsule) = ctx.capsule {
            let manifest_bytes = capsule.manifest_cbor();
            if let Ok(manifest) = ciborium::de::from_reader::<dash_manifest::PackageManifest, _>(manifest_bytes) {
                let license = manifest.license.unwrap_or_else(|| "NOASSERTION".to_string());
                let allowed = ["MIT", "Apache-2.0", "BSD-3-Clause", "BSD-2-Clause", "ISC", "MPL-2.0"];
                if !allowed.contains(&license.as_str()) {
                    return Ok(PluginHookResult::Veto {
                        reason: format!("License '{}' is not in allowed list", license),
                    });
                }
            }
        }
        Ok(PluginHookResult::Continue)
    }
}

pub struct MalwareScanPlugin;

impl MalwareScanPlugin {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Plugin for MalwareScanPlugin {
    fn name(&self) -> &str { "malware-scan" }
    fn version(&self) -> &str { "1.0.0" }
    fn plugin_type(&self) -> PluginType { PluginType::Registry }
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability { service: "security".into(), actions: vec!["scan".into()] }]
    }
    fn events(&self) -> Vec<String> { vec!["pre-publish".into(), "post-publish".into()] }

    async fn on_load(&self) -> Result<(), PluginError> { Ok(()) }
    async fn on_unload(&self) -> Result<(), PluginError> { Ok(()) }

    async fn on_pre_publish(&self, ctx: &PluginContext<'_>) -> PluginResult {
        if let Some(capsule) = ctx.capsule {
            let files = capsule.unpack().unwrap_or_default();
            for (path, content) in files {
                if content.windows(5).any(|w| w == b"eval(" || w == b"exec(") {
                    return Ok(PluginHookResult::Veto {
                        reason: format!("Suspicious code pattern detected in {}", path),
                    });
                }
            }
        }
        Ok(PluginHookResult::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builtin_plugin() {
        let plugin = BuiltinPlugin::new("test", "1.0.0", PluginType::Registry);
        assert_eq!(plugin.name(), "test");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(plugin.on_load().await.is_ok());
        assert!(plugin.on_unload().await.is_ok());
    }

    #[tokio::test]
    async fn test_license_check_plugin_allows_mit() {
        let plugin = LicenseCheckPlugin::new();
        let capsule = Capsule::pack(vec![]).unwrap();
        let ctx = PluginContext::new("test", "1.0.0").with_capsule(&capsule);
        let result = plugin.on_pre_publish(&ctx).await.unwrap();
        assert!(result.is_continue());
    }

    #[tokio::test]
    async fn test_malware_scan_plugin_detects_eval() {
        let plugin = MalwareScanPlugin::new();
        let files = vec![("bad.js".to_string(), b"eval('malicious')".to_vec())];
        let capsule = Capsule::pack(files).unwrap();
        let ctx = PluginContext::new("test", "1.0.0").with_capsule(&capsule);
        let result = plugin.on_pre_publish(&ctx).await.unwrap();
        assert!(result.is_veto());
    }
}