//! Error diagnostics, reporting, and structured diagnostic data.
//!
//! Provides infrastructure for collecting system information,
//! reporting diagnostic issues, and suggesting remediation steps.

use std::path::PathBuf;
use std::time::Duration;

/// Error type for diagnostics operations.
#[derive(Debug, thiserror::Error)]
pub enum DiagnosticError {
    #[error("collection error: {0}")]
    Collection(String),
    #[error("report error: {0}")]
    Report(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Severity levels for diagnostic issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum DiagnosticSeverity {
    /// Informational message.
    Info,
    /// Warning that may require attention.
    Warning,
    /// Error that should be addressed.
    Error,
    /// Critical error requiring immediate action.
    Critical,
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiagnosticSeverity::Info => write!(f, "INFO"),
            DiagnosticSeverity::Warning => write!(f, "WARNING"),
            DiagnosticSeverity::Error => write!(f, "ERROR"),
            DiagnosticSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A structured diagnostic issue with suggested remediation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticIssue {
    /// Unique error code for this issue.
    pub code: String,
    /// Severity level.
    pub severity: DiagnosticSeverity,
    /// Human-readable message describing the issue.
    pub message: String,
    /// Additional details or context.
    pub details: Option<String>,
    /// Suggested action to remediate the issue.
    pub suggested_action: Option<String>,
    /// Source location where the issue was detected.
    pub source: Option<String>,
    /// Timestamp when the issue was created.
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl DiagnosticIssue {
    /// Create a new diagnostic issue.
    pub fn new(
        code: impl Into<String>,
        severity: DiagnosticSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            details: None,
            suggested_action: None,
            source: None,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add details to this issue.
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add a suggested action to this issue.
    pub fn with_suggested_action(mut self, action: impl Into<String>) -> Self {
        self.suggested_action = Some(action.into());
        self
    }

    /// Add a source location to this issue.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

/// System information snapshot for diagnostic purposes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    /// Operating system name and version.
    pub os: String,
    /// CPU information.
    pub cpu: String,
    /// Number of logical CPU cores.
    pub cpu_cores: u32,
    /// Total physical memory in bytes.
    pub memory_total: u64,
    /// Available memory in bytes.
    pub memory_available: u64,
    /// Total disk space of root partition in bytes.
    pub disk_total: u64,
    /// Available disk space in bytes.
    pub disk_available: u64,
    /// Bnpm version.
    pub bnpm_version: String,
    /// Rust version (if known).
    pub rust_version: Option<String>,
    /// Hostname.
    pub hostname: Option<String>,
    /// Uptime in seconds.
    pub uptime_secs: Option<u64>,
}

/// Collect system information for diagnostic purposes.
pub fn collect_system_info() -> Result<SystemInfo, DiagnosticError> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH;

    let cpu_info = format!("{} ({} cores)", arch, num_cpus::get());
    let cpu_cores = num_cpus::get() as u32;

    // Memory info (best-effort)
    let (memory_total, memory_available) = get_memory_info();

    // Disk info (best-effort)
    let (disk_total, disk_available) = get_disk_info();

    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("HOST"))
        .ok();

    Ok(SystemInfo {
        os: format!("{} {}", os, arch),
        cpu: cpu_info,
        cpu_cores,
        memory_total,
        memory_available,
        disk_total,
        disk_available,
        bnpm_version: env!("CARGO_PKG_VERSION").to_string(),
        rust_version: Some(format!("{}", rustc_version())),
        hostname,
        uptime_secs: None,
    })
}

fn get_memory_info() -> (u64, u64) {
    // Use system-specific memory detection
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("sysctl").args(&["-n", "hw.memsize"]).output() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Ok(total) = s.trim().parse::<u64>() {
                    // Approximate available memory (simplified)
                    let available = total / 4; // rough estimate
                    return (total, available);
                }
            }
        }
    }
    (0, 0)
}

fn get_disk_info() -> (u64, u64) {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("df")
            .args(&["-k", "/"])
            .output()
        {
            let s = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = s.lines().collect();
            if lines.len() > 1 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 4 {
                    if let (Ok(total), Ok(avail)) = (parts[1].parse::<u64>(), parts[3].parse::<u64>()) {
                        return (total * 1024, avail * 1024);
                    }
                }
            }
        }
    }
    (0, 0)
}

fn rustc_version() -> String {
    option_env!("CARGO_PKG_RUST_VERSION")
        .unwrap_or("unknown")
        .to_string()
}

/// Number of logical CPUs.
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
    }
}

/// Report a diagnostic issue to the configured output.
pub fn report_diagnostic(issue: &DiagnosticIssue) -> Result<(), DiagnosticError> {
    let report = serde_json::to_string_pretty(issue)
        .map_err(|e| DiagnosticError::Report(e.to_string()))?;

    tracing::error!(
        target: "bnpm.diagnostics",
        code = %issue.code,
        severity = %issue.severity,
        "{}", issue.message
    );

    // Write to diagnostics log
    if let Some(config_dir) = dirs::config_dir() {
        let log_dir = config_dir.join("bnpm").join("diagnostics");
        std::fs::create_dir_all(&log_dir)?;
        let file_path = log_dir.join(format!("{}_{}.json", issue.code, chrono::Utc::now().timestamp()));
        std::fs::write(&file_path, &report)?;
    }

    Ok(())
}

/// A collector for aggregating multiple diagnostic issues.
#[derive(Debug, Default)]
pub struct DiagnosticsCollector {
    issues: Vec<DiagnosticIssue>,
}

impl DiagnosticsCollector {
    /// Create a new diagnostics collector.
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
        }
    }

    /// Add an issue to the collector.
    pub fn add(&mut self, issue: DiagnosticIssue) {
        self.issues.push(issue);
    }

    /// Report all collected issues.
    pub fn report_all(&self) -> Result<(), DiagnosticError> {
        for issue in &self.issues {
            report_diagnostic(issue)?;
        }
        Ok(())
    }

    /// Return the number of issues with the given or higher severity.
    pub fn count_by_severity(&self, min_severity: DiagnosticSeverity) -> usize {
        self.issues.iter().filter(|i| i.severity >= min_severity).count()
    }

    /// Get all issues.
    pub fn issues(&self) -> &[DiagnosticIssue] {
        &self.issues
    }

    /// Clear all issues.
    pub fn clear(&mut self) {
        self.issues.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_issue_new() {
        let issue = DiagnosticIssue::new("BNPM-001", DiagnosticSeverity::Error, "something went wrong");
        assert_eq!(issue.code, "BNPM-001");
        assert_eq!(issue.severity, DiagnosticSeverity::Error);
        assert_eq!(issue.message, "something went wrong");
    }

    #[test]
    fn test_diagnostic_issue_builder() {
        let issue = DiagnosticIssue::new("BNPM-002", DiagnosticSeverity::Warning, "disk space low")
            .with_details("less than 1GB available")
            .with_suggested_action("free up disk space");
        assert_eq!(issue.details.unwrap(), "less than 1GB available");
        assert_eq!(issue.suggested_action.unwrap(), "free up disk space");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(DiagnosticSeverity::Info < DiagnosticSeverity::Warning);
        assert!(DiagnosticSeverity::Warning < DiagnosticSeverity::Error);
        assert!(DiagnosticSeverity::Error < DiagnosticSeverity::Critical);
    }

    #[test]
    fn test_diagnostics_collector() {
        let mut collector = DiagnosticsCollector::new();
        collector.add(DiagnosticIssue::new("BNPM-001", DiagnosticSeverity::Info, "info"));
        collector.add(DiagnosticIssue::new("BNPM-002", DiagnosticSeverity::Error, "error"));
        assert_eq!(collector.issues().len(), 2);
        assert_eq!(collector.count_by_severity(DiagnosticSeverity::Error), 1);
        assert_eq!(collector.count_by_severity(DiagnosticSeverity::Info), 2);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(DiagnosticSeverity::Info.to_string(), "INFO");
        assert_eq!(DiagnosticSeverity::Warning.to_string(), "WARNING");
        assert_eq!(DiagnosticSeverity::Error.to_string(), "ERROR");
        assert_eq!(DiagnosticSeverity::Critical.to_string(), "CRITICAL");
    }

    #[test]
    fn test_collect_system_info() {
        let info = collect_system_info().unwrap();
        assert!(!info.os.is_empty());
        assert!(!info.cpu.is_empty());
        assert!(info.cpu_cores > 0);
        assert!(!info.bnpm_version.is_empty());
    }
}
