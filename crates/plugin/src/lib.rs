//! Dynamic plugin system for extending Dash via shared libraries.
//!
//! Provides a trait-based plugin interface and a manager for
//! loading, unloading, and accessing plugins at runtime.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Error type for plugin operations.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("plugin not found: {0}")]
    NotFound(String),
    #[error("plugin already loaded: {0}")]
    AlreadyLoaded(String),
    #[error("failed to load plugin from {path}: {message}")]
    LoadFailed { path: String, message: String },
    #[error("failed to unload plugin: {0}")]
    UnloadFailed(String),
    #[error("plugin initialization failed: {0}")]
    InitFailed(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("library error: {0}")]
    Library(String),
    #[error("incompatible plugin ABI: {0}")]
    IncompatibleAbi(String),
}

/// Plugin manifest metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Human-readable description.
    pub description: String,
    /// Plugin author.
    pub author: String,
    /// Entry point (library path or script path).
    pub entry: String,
    /// Plugin dependencies.
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Minimum Dash version required.
    #[serde(default)]
    pub min_dash_version: Option<String>,
    /// Plugin homepage.
    #[serde(default)]
    pub homepage: Option<String>,
    /// Plugin license.
    #[serde(default)]
    pub license: Option<String>,
}

impl PluginManifest {
    /// Validate the manifest.
    pub fn validate(&self) -> Result<(), PluginError> {
        if self.name.is_empty() {
            return Err(PluginError::InitFailed("plugin name is required".to_string()));
        }
        if self.entry.is_empty() {
            return Err(PluginError::InitFailed("plugin entry point is required".to_string()));
        }
        Ok(())
    }

    /// Load a manifest from a JSON file.
    pub fn load(path: &Path) -> Result<Self, PluginError> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Self = serde_json::from_str(&content)
            .map_err(|e| PluginError::InitFailed(format!("invalid manifest: {e}")))?;
        manifest.validate()?;
        Ok(manifest)
    }
}

/// The trait that all Dash plugins must implement.
pub trait DashPlugin: Send + Sync {
    /// Return the plugin name.
    fn name(&self) -> &str;
    /// Return the plugin version.
    fn version(&self) -> &str;
    /// Called when the plugin is loaded.
    fn on_load(&self) -> Result<(), PluginError>;
    /// Called when the plugin is unloaded.
    fn on_unload(&self) -> Result<(), PluginError>;
    /// Called when a package is being installed (hook).
    fn on_install_start(&self, _pkg_name: &str, _version: &str) -> Result<(), PluginError> {
        Ok(())
    }
    /// Called after a package is installed (hook).
    fn on_install_complete(&self, _pkg_name: &str, _version: &str) -> Result<(), PluginError> {
        Ok(())
    }
    /// Called when a package is being published (hook).
    fn on_publish_start(&self, _pkg_name: &str, _version: &str) -> Result<(), PluginError> {
        Ok(())
    }
    /// Called after a package is published (hook).
    fn on_publish_complete(&self, _pkg_name: &str, _version: &str) -> Result<(), PluginError> {
        Ok(())
    }
    /// Return custom commands provided by this plugin.
    fn commands(&self) -> Vec<PluginCommand> {
        Vec::new()
    }
}

/// A command registered by a plugin.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginCommand {
    /// Command name (e.g., "myplugin:do-something").
    pub name: String,
    /// Command description.
    pub description: String,
    /// Command usage information.
    pub usage: String,
}

impl PluginCommand {
    /// Create a new plugin command.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            usage: String::new(),
        }
    }

    /// Set the usage string.
    pub fn with_usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }
}

/// Manages loading, unloading, and access to plugins.
pub struct PluginManager {
    plugins: HashMap<String, Arc<dyn DashPlugin>>,
    manifests: HashMap<String, PluginManifest>,
}

impl PluginManager {
    /// Create a new empty plugin manager.
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            manifests: HashMap::new(),
        }
    }

    /// Load a plugin from a dynamic library path.
    pub fn load(&mut self, path: &Path) -> Result<(), PluginError> {
        // Validate the path
        if !path.exists() {
            return Err(PluginError::LoadFailed {
                path: path.to_string_lossy().to_string(),
                message: "file not found".to_string(),
            });
        }

        // Look for a manifest file alongside the library
        let manifest_dir = path.parent().unwrap_or(Path::new("."));
        let manifest_file = manifest_dir.join("plugin.json");
        let manifest = if manifest_file.exists() {
            PluginManifest::load(&manifest_file)?
        } else {
            return Err(PluginError::LoadFailed {
                path: path.to_string_lossy().to_string(),
                message: "plugin.json manifest not found".to_string(),
            });
        };

        if self.plugins.contains_key(&manifest.name) {
            return Err(PluginError::AlreadyLoaded(manifest.name));
        }

        // Load the dynamic library
        let plugin = unsafe {
            let lib = libloading::Library::new(path)
                .map_err(|e| PluginError::Library(e.to_string()))?;

            let constructor: libloading::Symbol<fn() -> Box<dyn DashPlugin>> = lib
                .get(b"create_plugin")
                .map_err(|e| PluginError::Library(format!("symbol 'create_plugin' not found: {e}")))?;

            constructor()
        };

        plugin.on_load()?;
        self.plugins.insert(manifest.name.clone(), Arc::from(plugin));
        self.manifests.insert(manifest.name.clone(), manifest);

        Ok(())
    }

    /// Unload a plugin by name.
    pub fn unload(&mut self, name: &str) -> Result<(), PluginError> {
        let plugin = self
            .plugins
            .remove(name)
            .ok_or_else(|| PluginError::NotFound(name.to_string()))?;

        plugin.on_unload()?;
        self.manifests.remove(name);
        Ok(())
    }

    /// Get a reference to a loaded plugin by name.
    pub fn get(&self, name: &str) -> Option<&dyn DashPlugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Check if a plugin is loaded.
    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }

    /// Get the manifest for a loaded plugin.
    pub fn manifest(&self, name: &str) -> Option<&PluginManifest> {
        self.manifests.get(name)
    }

    /// Return all loaded plugin names.
    pub fn loaded_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Return all loaded manifests.
    pub fn manifests(&self) -> Vec<&PluginManifest> {
        self.manifests.values().collect()
    }

    /// Unload all plugins.
    pub fn unload_all(&mut self) {
        let names: Vec<String> = self.plugins.keys().cloned().collect();
        for name in names {
            let _ = self.unload(&name);
        }
    }

    /// Get all commands registered by all loaded plugins.
    pub fn all_commands(&self) -> Vec<PluginCommand> {
        let mut commands = Vec::new();
        for plugin in self.plugins.values() {
            commands.extend(plugin.commands());
        }
        commands
    }

    /// Execute a plugin command.
    pub fn execute_command(&self, _name: &str, _args: &[String]) -> Result<(), PluginError> {
        // This would look up the command and execute it through the plugin
        Err(PluginError::NotFound("command execution not yet implemented".to_string()))
    }

    /// Trigger the on_load hook for all plugins.
    pub fn notify_load_all(&self) -> Result<(), PluginError> {
        for plugin in self.plugins.values() {
            plugin.on_load()?;
        }
        Ok(())
    }

    /// Trigger the on_unload hook for all plugins.
    pub fn notify_unload_all(&self) -> Result<(), PluginError> {
        for plugin in self.plugins.values() {
            plugin.on_unload()?;
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// A simple plugin that can be created inline without dynamic loading.
pub struct SimplePlugin {
    name: String,
    version: String,
}

impl SimplePlugin {
    /// Create a new simple plugin.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

impl DashPlugin for SimplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn on_load(&self) -> Result<(), PluginError> {
        tracing::info!("Plugin '{}' loaded (v{})", self.name, self.version);
        Ok(())
    }

    fn on_unload(&self) -> Result<(), PluginError> {
        tracing::info!("Plugin '{}' unloaded", self.name);
        Ok(())
    }
}

/// Load all plugins from a directory.
pub fn load_plugins_from_dir(manager: &mut PluginManager, dir: &Path) -> Result<Vec<String>, PluginError> {
    let mut loaded = Vec::new();

    if !dir.exists() {
        return Ok(loaded);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Look for shared libraries
        let is_lib = path.extension().map_or(false, |ext| {
            matches!(ext.to_str(), Some("so" | "dylib" | "dll"))
        });

        if is_lib {
            match manager.load(&path) {
                Ok(()) => {
                    if let Some(manifest) = manager.manifest(&path.to_string_lossy()) {
                        loaded.push(manifest.name.clone());
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to load plugin from {}: {e}", path.display());
                }
            }
        }
    }

    Ok(loaded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_plugin() {
        let plugin = SimplePlugin::new("test-plugin", "1.0.0");
        assert_eq!(plugin.name(), "test-plugin");
        assert_eq!(plugin.version(), "1.0.0");
        assert!(plugin.on_load().is_ok());
        assert!(plugin.on_unload().is_ok());
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        assert!(manager.loaded_plugins().is_empty());

        // Add a plugin directly (simulating loading)
        let plugin = Arc::new(SimplePlugin::new("inline-plugin", "1.0.0")) as Arc<dyn DashPlugin>;
        // We can't use the typical load() since it requires a file path,
        // but we can insert directly into the hashmap for testing
        manager.plugins.insert("inline-plugin".to_string(), plugin);

        assert!(manager.is_loaded("inline-plugin"));
        assert_eq!(manager.loaded_plugins().len(), 1);
    }

    #[test]
    fn test_plugin_command() {
        let cmd = PluginCommand::new("test:hello", "Prints hello")
            .with_usage("test:hello [name]");
        assert_eq!(cmd.name, "test:hello");
        assert_eq!(cmd.usage, "test:hello [name]");
    }

    #[test]
    fn test_plugin_manifest() {
        let manifest = PluginManifest {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: "Dash Team".to_string(),
            entry: "libtest.so".to_string(),
            dependencies: Vec::new(),
            min_dash_version: None,
            homepage: None,
            license: None,
        };

        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_plugin_manifest_invalid() {
        let manifest = PluginManifest {
            name: String::new(),
            version: "1.0.0".to_string(),
            description: String::new(),
            author: String::new(),
            entry: String::new(),
            dependencies: Vec::new(),
            min_dash_version: None,
            homepage: None,
            license: None,
        };

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_plugin_hooks() {
        let plugin = SimplePlugin::new("hook-test", "1.0.0");
        assert!(plugin.on_install_start("pkg", "1.0.0").is_ok());
        assert!(plugin.on_install_complete("pkg", "1.0.0").is_ok());
        assert!(plugin.on_publish_start("pkg", "1.0.0").is_ok());
        assert!(plugin.on_publish_complete("pkg", "1.0.0").is_ok());
    }

    #[test]
    fn test_plugin_manager_unload_all() {
        let mut manager = PluginManager::new();
        let plugin = Arc::new(SimplePlugin::new("p1", "1.0.0")) as Arc<dyn DashPlugin>;
        manager.plugins.insert("p1".to_string(), plugin);
        let plugin = Arc::new(SimplePlugin::new("p2", "1.0.0")) as Arc<dyn DashPlugin>;
        manager.plugins.insert("p2".to_string(), plugin);

        assert_eq!(manager.loaded_plugins().len(), 2);
        manager.unload_all();
        assert!(manager.loaded_plugins().is_empty());
    }

    #[test]
    fn test_load_plugins_from_dir_nonexistent() {
        let mut manager = PluginManager::new();
        let result = load_plugins_from_dir(&mut manager, Path::new("/nonexistent/plugins"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
