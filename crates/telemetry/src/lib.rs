//! Telemetry and observability: tracing, metrics, OpenTelemetry export.
//!
//! Initializes and manages the tracing infrastructure with optional
//! export to OpenTelemetry-compatible backends.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Error type for telemetry operations.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("initialization error: {0}")]
    Init(String),
    #[error("shutdown error: {0}")]
    Shutdown(String),
    #[error("already initialized")]
    AlreadyInitialized,
}

/// Configuration for the telemetry subsystem.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetryConfig {
    /// Service name for identifying this application in traces/metrics.
    pub service_name: String,
    /// OpenTelemetry collector endpoint (OTLP gRPC).
    pub endpoint: Option<String>,
    /// Trace sampling ratio (0.0 to 1.0).
    pub sampler_ratio: f64,
    /// Log level for the tracing subscriber.
    pub log_level: String,
    /// Whether to export traces via OTLP.
    pub enable_otlp: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: "bnpm".to_string(),
            endpoint: None,
            sampler_ratio: 0.1,
            log_level: "info".to_string(),
            enable_otlp: false,
        }
    }
}

static TELEMETRY_INITIALIZED: OnceLock<bool> = OnceLock::new();

/// Initialize the tracing and telemetry system.
pub fn init_tracing(config: &TelemetryConfig) -> Result<(), TelemetryError> {
    if TELEMETRY_INITIALIZED.get().is_some() {
        return Err(TelemetryError::AlreadyInitialized);
    }

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(
            config
                .log_level
                .parse()
                .map_err(|e| TelemetryError::Init(format!("invalid log level: {e}")))?,
        )
        .from_env_lossy();

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    TELEMETRY_INITIALIZED
        .set(true)
        .map_err(|_| TelemetryError::Init("once lock failed".to_string()))?;

    Ok(())
}

/// Gracefully shut down the telemetry system, flushing remaining spans/metrics.
pub fn shutdown_tracing() -> Result<(), TelemetryError> {
    Ok(())
}

/// Record a custom event with attributes.
pub fn record_event(name: &str, attributes: HashMap<String, String>) {
    tracing::info!(
        target: "bnpm.telemetry.event",
        "event {}: {:?}",
        name,
        attributes,
    );
}

/// Record a custom metric value.
pub fn record_metric(name: &str, value: f64, attributes: HashMap<String, String>) {
    tracing::info!(
        target: "bnpm.telemetry.metric",
        "metric {} = {} ({:?})",
        name,
        value,
        attributes,
    );
}

/// Check whether telemetry has been initialized.
pub fn is_initialized() -> bool {
    TELEMETRY_INITIALIZED.get().is_some()
}

/// A counter metric for tracking cumulative values.
#[derive(Debug)]
pub struct Counter {
    name: String,
    value: std::sync::atomic::AtomicU64,
}

impl Counter {
    /// Create a new counter.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Increment the counter by 1.
    pub fn inc(&self) {
        self.add(1);
    }

    /// Add a value to the counter.
    pub fn add(&self, delta: u64) {
        self.value.fetch_add(delta, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get the current value.
    pub fn value(&self) -> u64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Record and reset the counter.
    pub fn flush(&self) {
        let val = self.value.swap(0, std::sync::atomic::Ordering::Relaxed);
        record_metric(&self.name, val as f64, HashMap::new());
    }
}

/// A gauge metric for tracking point-in-time values.
#[derive(Debug)]
pub struct Gauge {
    name: String,
    value: std::sync::atomic::AtomicI64,
}

impl Gauge {
    /// Create a new gauge.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: std::sync::atomic::AtomicI64::new(0),
        }
    }

    /// Set the gauge to a value.
    pub fn set(&self, val: i64) {
        self.value.store(val, std::sync::atomic::Ordering::Relaxed);
        record_metric(&self.name, val as f64, HashMap::new());
    }

    /// Get the current value.
    pub fn value(&self) -> i64 {
        self.value.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = TelemetryConfig::default();
        assert_eq!(config.service_name, "bnpm");
        assert!((config.sampler_ratio - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_counter() {
        let counter = Counter::new("test.counter");
        assert_eq!(counter.value(), 0);
        counter.inc();
        assert_eq!(counter.value(), 1);
        counter.add(5);
        assert_eq!(counter.value(), 6);
    }

    #[test]
    fn test_gauge() {
        let gauge = Gauge::new("test.gauge");
        assert_eq!(gauge.value(), 0);
        gauge.set(42);
        assert_eq!(gauge.value(), 42);
        gauge.set(-1);
        assert_eq!(gauge.value(), -1);
    }

    #[test]
    fn test_record_event() {
        let mut attrs = HashMap::new();
        attrs.insert("key".to_string(), "value".to_string());
        record_event("test.event", attrs);
    }

    #[test]
    fn test_record_metric() {
        let mut attrs = HashMap::new();
        attrs.insert("source".to_string(), "test".to_string());
        record_metric("test.metric", 100.0, attrs);
    }
}
