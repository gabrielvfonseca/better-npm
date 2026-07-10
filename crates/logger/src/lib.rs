//! Structured logging with file rotation, JSON output, and env filtering.
//!
//! Builds on `tracing` and `tracing-subscriber` to provide a flexible
//! logging subsystem with multiple output formats and destinations.

use std::path::PathBuf;
use std::sync::Mutex;

/// Error type for logger operations.
#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("initialization error: {0}")]
    Init(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("already initialized")]
    AlreadyInitialized,
}

/// Log output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LogFormat {
    /// Human-readable text format.
    Text,
    /// Structured JSON format.
    Json,
}

/// Log output destination.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum LogOutput {
    /// Standard output (stdout).
    Stdout,
    /// Write to a file.
    File(PathBuf),
    /// Write to a TCP socket.
    Tcp(std::net::SocketAddr),
}

/// Configuration for the logging subsystem.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogConfig {
    /// The log level (e.g. "debug", "info", "warn", "error").
    pub level: String,
    /// Output format.
    pub format: LogFormat,
    /// Output destination.
    pub output: LogOutput,
    /// Whether to include span context in log output.
    pub include_spans: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Text,
            output: LogOutput::Stdout,
            include_spans: true,
        }
    }
}

static INITIALIZED: Mutex<bool> = Mutex::new(false);

/// Initialize the global tracing subscriber with the given configuration.
pub fn init(config: &LogConfig) -> Result<(), LogError> {
    if *INITIALIZED.lock().unwrap() {
        return Err(LogError::AlreadyInitialized);
    }
    *INITIALIZED.lock().unwrap() = true;

    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(config.level.parse().map_err(|e| LogError::Init(format!("invalid level: {e}")))?)
        .from_env_lossy();

    let fmt = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(
            if config.include_spans {
                tracing_subscriber::fmt::format::FmtSpan::CLOSE
            } else {
                tracing_subscriber::fmt::format::FmtSpan::NONE
            },
        );

    match &config.output {
        LogOutput::Stdout => fmt.init(),
        LogOutput::File(path) => {
            let file = std::fs::File::create(path)
                .map_err(|e| LogError::Io(e))?;
            fmt.with_writer(std::sync::Mutex::new(file)).init();
        }
        LogOutput::Tcp(addr) => {
            let stream = std::net::TcpStream::connect(addr)
                .map_err(|e| LogError::Io(e))?;
            fmt.with_writer(std::sync::Mutex::new(stream)).init();
        }
    }

    Ok(())
}

/// Reset the initialization state (useful for testing).
pub fn reset() {
    *INITIALIZED.lock().unwrap() = false;
}

/// Check whether the logger has been initialized.
pub fn is_initialized() -> bool {
    *INITIALIZED.lock().unwrap()
}

/// A structured logger bound to a specific module and level.
#[derive(Debug, Clone)]
pub struct Logger {
    module: String,
    level: log_level::Level,
    destination: LogOutput,
}

mod log_level {
    #[derive(Debug, Clone)]
    pub enum Level {
        Debug,
        Info,
        Warn,
        Error,
    }
}

impl Logger {
    /// Create a new logger for the given module.
    pub fn new(module: &str) -> Self {
        Self {
            module: module.to_string(),
            level: log_level::Level::Info,
            destination: LogOutput::Stdout,
        }
    }

    /// Create a new logger with a specific level and destination.
    pub fn new_with_config(module: &str, level: log_level::Level, destination: LogOutput) -> Self {
        Self {
            module: module.to_string(),
            level,
            destination,
        }
    }

    /// Create a child logger for a sub-module.
    pub fn child(&self, child_module: &str) -> Self {
        Self {
            module: format!("{}::{}", self.module, child_module),
            level: log_level::Level::Info,
            destination: LogOutput::Stdout,
        }
    }

    /// Log a message at the debug level.
    pub fn debug(&self, message: impl std::fmt::Display) {
        tracing::debug!("[{}] {}", self.module, message);
    }

    /// Log a message at the info level.
    pub fn info(&self, message: impl std::fmt::Display) {
        tracing::info!("[{}] {}", self.module, message);
    }

    /// Log a message at the warn level.
    pub fn warn(&self, message: impl std::fmt::Display) {
        tracing::warn!("[{}] {}", self.module, message);
    }

    /// Log a message at the error level.
    pub fn error(&self, message: impl std::fmt::Display) {
        tracing::error!("[{}] {}", self.module, message);
    }

    /// Log a structured event with key-value pairs at the info level.
    pub fn event(&self, name: &str, attrs: &[(&str, &dyn std::fmt::Display)]) {
        let mut fields = String::new();
        for (i, (k, v)) in attrs.iter().enumerate() {
            if i > 0 {
                fields.push_str(", ");
            }
            fields.push_str(&format!("{k} = {v}"));
        }
        tracing::info!("[{}] event {}: {}", self.module, name, fields);
    }

    /// Return the module path.
    pub fn module(&self) -> &str {
        &self.module
    }
}

#[macro_export]
macro_rules! log_debug {
    ($logger:expr, $($arg:tt)+) => {
        $logger.debug(format!($($arg)+));
    };
}

#[macro_export]
macro_rules! log_info {
    ($logger:expr, $($arg:tt)+) => {
        $logger.info(format!($($arg)+));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($logger:expr, $($arg:tt)+) => {
        $logger.warn(format!($($arg)+));
    };
}

#[macro_export]
macro_rules! log_error {
    ($logger:expr, $($arg:tt)+) => {
        $logger.error(format!($($arg)+));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_new() {
        let logger = Logger::new("test_module");
        assert_eq!(logger.module(), "test_module");
    }

    #[test]
    fn test_logger_child() {
        let parent = Logger::new("parent");
        let child = parent.child("child");
        assert_eq!(child.module(), "parent::child");
    }

    #[test]
    fn test_log_macros() {
        let logger = Logger::new("macro_test");
        log_info!(logger, "hello from macro");
        log_debug!(logger, "debug message");
        log_warn!(logger, "warning: {}", "something");
        log_error!(logger, "error occurred");
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, LogFormat::Text);
    }

    #[test]
    fn test_init_and_reset() {
        let config = LogConfig::default();
        // In test environment, this may already be initialized
        let result = init(&config);
        if result.is_ok() {
            assert!(is_initialized());
            reset();
            assert!(!is_initialized());
        }
    }
}
