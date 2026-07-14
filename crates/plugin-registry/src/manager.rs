use std::path::Path;
use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::info;

use crate::plugins::{Plugin, PluginContext, PluginError, PluginHookResult, PluginManifest, PluginMetadata, PluginType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManagerConfig {
    pub plugin_dir: Option<String>,
    pub enable_sandbox: bool,
    pub max_concurrent_plugins: usize,
    pub plugin_timeout_secs: u64,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            plugin_dir: Some("./plugins".to_string()),
            enable_sandbox: true,
            max_concurrent_plugins: 10,
            plugin_timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEvent {
    pub event_type: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub plugin_name: String,
    pub result: PluginHookResult,
    pub context: serde_json::Value,
}

pub struct PluginManager {
    config: PluginManagerConfig,
    plugins: DashMap<String, Arc<dyn Plugin>>,
    manifests: DashMap<String, PluginManifest>,
    metadata: DashMap<String, PluginMetadata>,
    event_log: Arc<RwLock<Vec<PluginEvent>>>,
}

impl PluginManager {
    pub fn new(config: PluginManagerConfig) -> Self {
        Self {
            config,
            plugins: DashMap::new(),
            manifests: DashMap::new(),
            metadata: DashMap::new(),
            event_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn load_plugin(&self, path: &Path) -> Result<String, PluginError> {
        let manifest = PluginManifest::load(&path.join("plugin.json"))?;
        let name = manifest.name.clone();

        if self.plugins.contains_key(&name) {
            return Err(PluginError::AlreadyLoaded(name));
        }

        let plugin = self.load_plugin_from_path(&path, &manifest).await?;
        plugin.on_load().await?;

        let metadata = PluginMetadata {
            manifest: manifest.clone(),
            loaded_at: chrono::Utc::now(),
            path: path.display().to_string(),
            hash: compute_hash(&path)?,
        };

        self.plugins.insert(name.clone(), plugin);
        self.manifests.insert(name.clone(), manifest);
        self.metadata.insert(name.clone(), metadata);

        info!("Loaded plugin '{}'", name);
        Ok(name)
    }

    async fn load_plugin_from_path(&self, _path: &Path, manifest: &PluginManifest) -> Result<Arc<dyn Plugin>, PluginError> {
        match manifest.plugin_type {
            PluginType::Registry => {
                if manifest.name == "license-check" {
                    Ok(Arc::new(crate::plugins::LicenseCheckPlugin::new()))
                } else if manifest.name == "malware-scan" {
                    Ok(Arc::new(crate::plugins::MalwareScanPlugin::new()))
                } else {
                    Err(PluginError::InitFailed(format!("unknown builtin plugin: {}", manifest.name)))
                }
            }
            PluginType::Builder => Err(PluginError::InitFailed("builder plugins loaded via BuilderRegistry".into())),
            PluginType::Cli => Err(PluginError::InitFailed("CLI plugins loaded via CLI".into())),
        }
    }

    pub async fn register_plugin(&self, plugin: Arc<dyn Plugin>) -> Result<(), PluginError> {
        let name = plugin.name().to_string();
        if self.plugins.contains_key(&name) {
            return Err(PluginError::AlreadyLoaded(name));
        }
        plugin.on_load().await?;
        self.plugins.insert(name.clone(), plugin);
        info!("Registered plugin '{}'", name);
        Ok(())
    }

    pub async fn unload_plugin(&self, name: &str) -> Result<(), PluginError> {
        let (_, plugin) = self.plugins.remove(name).ok_or(PluginError::NotFound(name.to_string()))?;
        plugin.on_unload().await?;
        self.manifests.remove(name);
        self.metadata.remove(name);
        info!("Unloaded plugin '{}'", name);
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(name).map(|p| p.clone())
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    pub fn loaded_plugins(&self) -> Vec<String> {
        self.plugins.iter().map(|e| e.key().clone()).collect()
    }

    pub fn get_manifest(&self, name: &str) -> Option<PluginManifest> {
        self.manifests.get(name).map(|m| m.clone())
    }

    pub async fn run_hook(&self, event: &str, ctx: &PluginContext<'_>) -> Result<Vec<PluginHookResult>, PluginError> {
        let mut results = Vec::new();
        for entry in self.plugins.iter() {
            let plugin = entry.value();
            if plugin.events().contains(&event.to_string()) {
                let result = match event {
                    "pre-publish" => plugin.on_pre_publish(ctx).await?,
                    "post-publish" => plugin.on_post_publish(ctx).await?,
                    "pre-resolve" => plugin.on_pre_resolve(ctx).await?,
                    "post-resolve" => plugin.on_post_resolve(ctx).await?,
                    "pre-fetch" => plugin.on_pre_fetch(ctx).await?,
                    "post-fetch" => plugin.on_post_fetch(ctx).await?,
                    "policy-check" => plugin.on_policy_check(ctx).await?,
                    "version-added" => plugin.on_version_added(ctx).await?,
                    "yank" => plugin.on_yank(ctx).await?,
                    "tag-change" => plugin.on_tag_change(ctx).await?,
                    _ => PluginHookResult::Continue,
                };
                results.push(result.clone());

                self.log_event(PluginEvent {
                    event_type: event.to_string(),
                    timestamp: chrono::Utc::now(),
                    plugin_name: plugin.name().to_string(),
                    result: result.clone(),
                    context: serde_json::to_value(ctx.metadata.clone()).unwrap_or_default(),
                }).await;
            }
        }
        Ok(results)
    }

    async fn log_event(&self, event: PluginEvent) {
        let mut log = self.event_log.write().await;
        log.push(event);
        if log.len() > 10000 {
            log.drain(0..1000);
        }
    }

    pub async fn get_event_log(&self, limit: usize) -> Vec<PluginEvent> {
        let log = self.event_log.read().await;
        log.iter().rev().take(limit).cloned().collect()
    }

    pub async fn load_all_from_dir(&self, dir: &Path) -> Result<Vec<String>, PluginError> {
        let mut loaded = Vec::new();
        if !dir.exists() {
            return Ok(loaded);
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.join("plugin.json").exists() {
                if let Ok(name) = self.load_plugin(&path).await {
                    loaded.push(name);
                }
            }
        }
        Ok(loaded)
    }
}

fn compute_hash(path: &Path) -> Result<String, PluginError> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let content = std::fs::read(entry.path())?;
            hasher.update(&content);
        }
    }
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{BuiltinPlugin, PluginType};

    #[tokio::test]
    async fn test_plugin_manager_new() {
        let manager = PluginManager::new(PluginManagerConfig::default());
        assert!(manager.loaded_plugins().is_empty());
    }

    #[tokio::test]
    async fn test_register_and_unload_plugin() {
        let manager = PluginManager::new(PluginManagerConfig::default());
        let plugin = Arc::new(BuiltinPlugin::new("test", "1.0.0", PluginType::Registry));
        manager.register_plugin(plugin).await.unwrap();
        assert!(manager.is_loaded("test"));
        manager.unload_plugin("test").await.unwrap();
        assert!(!manager.is_loaded("test"));
    }

    #[tokio::test]
    async fn test_run_hooks() {
        let manager = PluginManager::new(PluginManagerConfig::default());
        let plugin = Arc::new(BuiltinPlugin::new("hook-test", "1.0.0", PluginType::Registry)
            .with_event("pre-publish"));
        manager.register_plugin(plugin).await.unwrap();

        let ctx = PluginContext::new("pkg", "1.0.0");
        let results = manager.run_hook("pre-publish", &ctx).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_continue());
    }
}