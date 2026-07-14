pub mod manager;
pub mod plugins;

pub use manager::{PluginManager, PluginManagerConfig, PluginEvent};
pub use plugins::{
    Plugin, PluginCapability, PluginContext, PluginError, PluginHookResult, PluginManifest,
    PluginMetadata, PluginType,
    BuiltinPlugin, LicenseCheckPlugin, MalwareScanPlugin,
};