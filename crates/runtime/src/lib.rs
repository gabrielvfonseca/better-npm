//! Application runtime initialization and lifecycle management.
//!
//! Provides a unified runtime that sets up tokio, logging, telemetry,
//! and configuration, and manages graceful shutdown.

use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Error type for runtime operations.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("runtime initialization failed: {0}")]
    Init(String),
    #[error("runtime shutdown failed: {0}")]
    Shutdown(String),
    #[error("configuration error: {0}")]
    Config(#[from] bnpm_config::ConfigError),
    #[error("telemetry error: {0}")]
    Telemetry(#[from] bnpm_telemetry::TelemetryError),
    #[error("logger error: {0}")]
    Logger(#[from] bnpm_logger::LogError),
}

/// Configuration for the Bnpm runtime.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuntimeConfig {
    /// Number of worker threads for the tokio runtime.
    pub worker_threads: Option<usize>,
    /// Maximum number of blocking threads.
    pub max_blocking_threads: Option<usize>,
    /// Enable I/O event loop.
    pub enable_io: bool,
    /// Enable time driver.
    pub enable_time: bool,
    /// Thread name prefix.
    pub thread_name: Option<String>,
    /// Stack size in bytes.
    pub stack_size: Option<usize>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: None,
            max_blocking_threads: None,
            enable_io: true,
            enable_time: true,
            thread_name: Some("bnpm-worker".to_string()),
            stack_size: None,
        }
    }
}

impl From<&bnpm_config::Config> for RuntimeConfig {
    fn from(config: &bnpm_config::Config) -> Self {
        Self {
            worker_threads: config.get("runtime.worker_threads").ok(),
            max_blocking_threads: config.get("runtime.max_blocking_threads").ok(),
            enable_io: config.get_or("runtime.enable_io", true),
            enable_time: config.get_or("runtime.enable_time", true),
            thread_name: config.get("runtime.thread_name").ok().or(Some("bnpm-worker".to_string())),
            stack_size: config.get("runtime.stack_size").ok(),
        }
    }
}

/// A handle to the Bnpm runtime, providing lifecycle management.
pub struct RuntimeHandle {
    /// The underlying tokio runtime.
    pub tokio_rt: Runtime,
    /// The runtime configuration.
    pub config: RuntimeConfig,
}

impl RuntimeHandle {
    /// Create a new runtime handle from a tokio runtime.
    pub fn new(tokio_rt: Runtime, config: RuntimeConfig) -> Self {
        Self { tokio_rt, config }
    }

    /// Initialize the full Bnpm runtime (tokio + logging + telemetry).
    pub fn init(config: &RuntimeConfig) -> Result<Self, RuntimeError> {
        let tokio_rt = build_tokio_runtime(config)?;

        // Initialize logging
        let log_config = bnpm_logger::LogConfig::default();
        // Don't fail if already initialized
        let _ = bnpm_logger::init(&log_config);

        Ok(Self::new(tokio_rt, config.clone()))
    }

    /// Initialize the runtime with a full application config.
    pub fn from_config(config: &bnpm_config::Config) -> Result<Self, RuntimeError> {
        let runtime_config = RuntimeConfig::from(config);
        Self::init(&runtime_config)
    }

    /// Block on a future, running it to completion on the tokio runtime.
    pub fn block_on<T>(&self, future: impl Future<Output = T>) -> T {
        self.tokio_rt.block_on(future)
    }

    /// Spawn a new task on the tokio runtime.
    pub fn spawn<T>(&self, future: impl Future<Output = T> + Send + 'static) -> tokio::task::JoinHandle<T>
    where
        T: Send + 'static + std::fmt::Debug,
    {
        self.tokio_rt.spawn(future)
    }

    /// Spawn a blocking task on the tokio runtime.
    pub fn spawn_blocking<T>(&self, f: impl FnOnce() -> T + Send + 'static) -> tokio::task::JoinHandle<T>
    where
        T: Send + 'static,
    {
        self.tokio_rt.spawn_blocking(f)
    }

    /// Enter the runtime context (for sync code that needs async).
    pub fn enter(&self) -> tokio::runtime::EnterGuard<'_> {
        self.tokio_rt.enter()
    }

    /// Shutdown the runtime gracefully.
    pub fn shutdown(self) -> Result<(), RuntimeError> {
        // Shutdown telemetry
        let _ = bnpm_telemetry::shutdown_tracing();
        Ok(())
    }

    /// Return a reference to the tokio runtime.
    pub fn runtime(&self) -> &Runtime {
        &self.tokio_rt
    }
}

fn build_tokio_runtime(config: &RuntimeConfig) -> Result<Runtime, RuntimeError> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();

    if let Some(threads) = config.worker_threads {
        builder.worker_threads(threads);
    }

    if let Some(threads) = config.max_blocking_threads {
        builder.max_blocking_threads(threads);
    }

    if let Some(name) = &config.thread_name {
        builder.thread_name(name);
    }

    if let Some(size) = config.stack_size {
        builder.thread_stack_size(size);
    }

    builder.enable_all();

    builder
        .build()
        .map_err(|e| RuntimeError::Init(e.to_string()))
}

/// Convenience function to block on a future using the global runtime.
///
/// Panics if called outside a tokio runtime context.
pub fn block_on<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Handle::current().block_on(future)
}

/// Spawn a task on the current runtime.
pub fn spawn<T>(future: impl Future<Output = T> + Send + 'static) -> tokio::task::JoinHandle<T>
where
    T: Send + 'static,
{
    tokio::spawn(future)
}

/// Initialize the default runtime and config.
pub fn init_default_runtime() -> Result<RuntimeHandle, RuntimeError> {
    let config = RuntimeConfig::default();
    RuntimeHandle::init(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert!(config.enable_io);
        assert!(config.enable_time);
    }

    #[test]
    fn test_build_tokio_runtime() {
        let config = RuntimeConfig::default();
        let runtime = build_tokio_runtime(&config).unwrap();
        runtime.block_on(async { assert!(tokio::runtime::Handle::try_current().is_ok()) });
    }

    #[test]
    fn test_runtime_handle_block_on() {
        let config = RuntimeConfig::default();
        let handle = RuntimeHandle::init(&config).unwrap();
        let result = handle.block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_runtime_handle_spawn() {
        let config = RuntimeConfig::default();
        let handle = RuntimeHandle::init(&config).unwrap();
        let task = handle.spawn(async { "hello from task" });
        let result = handle.block_on(task).unwrap();
        assert_eq!(result, "hello from task");
    }

    #[test]
    fn test_runtime_handle_spawn_blocking() {
        let config = RuntimeConfig::default();
        let handle = RuntimeHandle::init(&config).unwrap();
        let task = handle.spawn_blocking(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        let result = handle.block_on(task).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_init_default_runtime() {
        let handle = init_default_runtime().unwrap();
        let result = handle.block_on(async { true });
        assert!(result);
    }

    #[test]
    fn test_runtime_config_from_config() {
        let mut config = bnpm_config::Config::empty();
        config.set("runtime.worker_threads", 4);
        let runtime_config = RuntimeConfig::from(&config);
        assert_eq!(runtime_config.worker_threads, Some(4));
    }

    #[test]
    fn test_block_on_fn() {
        let config = RuntimeConfig::default();
        let runtime = build_tokio_runtime(&config).unwrap();
        let _guard = runtime.enter();
        let result = block_on(async { true });
        assert!(result);
    }

    #[test]
    fn test_shutdown_does_not_panic() {
        let config = RuntimeConfig::default();
        let handle = RuntimeHandle::init(&config).unwrap();
        assert!(handle.shutdown().is_ok());
    }
}
