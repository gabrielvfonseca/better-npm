//! Shared CLI configuration and types

use std::path::PathBuf;

/// Exit codes matching CLI.md specification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    Error = 1,
    PolicyViolation = 2,
    ResolutionConflict = 3,
    LockfileChanged = 4,
    NetworkRequired = 5,
    PermissionDenied = 6,
    NoUpdateAvailable = 7,
}

impl ExitCode {
    pub fn code(self) -> i32 {
        self as i32
    }
}

impl std::fmt::Display for ExitCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CliConfig {
    pub json_output: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub offline: bool,
    pub frozen_lock: bool,
    pub dry_run: bool,
    pub color: bool,
    pub no_color: bool,
    pub cache_dir: Option<PathBuf>,
    pub store_dir: Option<PathBuf>,
    pub registry: Vec<String>,
    pub config_path: Option<PathBuf>,
    pub why: bool,
}

impl CliConfig {
    pub fn load() -> Result<Self, anyhow::Error> {
        Ok(Self::default())
    }

    pub fn set_verbosity(&mut self, verbosity: u8) {
        self.verbose = verbosity > 0;
    }

    pub fn store_dir(&self) -> PathBuf {
        self.store_dir.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".dash")
                .join("store")
        })
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.cache_dir
            .clone()
            .unwrap_or_else(|| dirs::cache_dir().unwrap_or_else(|| PathBuf::from(".")).join("dash"))
    }

    pub fn registry_url(&self) -> String {
        self.registry
            .first()
            .cloned()
            .unwrap_or_else(|| "https://registry.dashjs.dev".to_string())
    }
}

/// Global arguments shared by all commands
#[derive(Debug, Clone, Default)]
pub struct Args {
    pub verbose: u8,
    pub quiet: bool,
    pub config: Option<PathBuf>,
    pub dry_run: bool,
    pub no_color: bool,
    pub color: bool,
    pub offline: bool,
    pub frozen_lock: bool,
    pub json: bool,
    pub why: bool,
    pub cache_dir: Option<PathBuf>,
    pub store_dir: Option<PathBuf>,
    pub registry: Vec<String>,
    pub help: bool,
}

/// Command-specific arguments
#[derive(Debug, Clone, Default)]
pub struct InitArgs {
    pub path: PathBuf,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub license: String,
    pub author: Option<String>,
    pub module_type: Option<String>,
    pub private: bool,
    pub ai: bool,
    pub yes: bool,
    pub no_import: bool,
    pub workspace: bool,
    pub bare: bool,
}
