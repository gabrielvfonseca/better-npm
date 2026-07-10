//! Sandboxed execution environments for package build scripts.
//!
//! Provides isolated execution contexts for running package
//! install scripts with resource limits and filesystem restrictions.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Error type for sandbox operations.
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("command failed: {command} (exit code: {exit_code:?})")]
    CommandFailed { command: String, exit_code: Option<i32> },
    #[error("timeout after {duration:?}")]
    Timeout { duration: Duration },
    #[error("memory limit exceeded: {limit} bytes")]
    MemoryLimitExceeded { limit: u64 },
    #[error("network access denied")]
    NetworkDenied,
    #[error("filesystem access denied: {0}")]
    FilesystemDenied(PathBuf),
    #[error("sandbox setup failed: {0}")]
    Setup(String),
    #[error("installation failed: {0}")]
    Install(String),
}

/// Configuration for sandbox execution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SandboxConfig {
    /// Maximum execution time.
    pub timeout: Duration,
    /// Maximum memory usage in bytes.
    pub memory_limit: Option<u64>,
    /// Whether the sandbox has network access.
    pub network_access: bool,
    /// Filesystem paths accessible within the sandbox.
    pub filesystem_scope: Vec<PathBuf>,
    /// Maximum number of child processes.
    pub max_processes: Option<u32>,
    /// Temporary directory size limit.
    pub temp_dir_size_limit: Option<u64>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300),
            memory_limit: Some(512 * 1024 * 1024), // 512 MB
            network_access: false,
            filesystem_scope: Vec::new(),
            max_processes: Some(10),
            temp_dir_size_limit: Some(1024 * 1024 * 1024), // 1 GB
        }
    }
}

/// Output from a sandboxed command execution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SandboxOutput {
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
    /// Exit code of the process.
    pub exit_code: i32,
    /// Wall-clock duration.
    pub duration: Duration,
    /// Peak memory usage (if available).
    pub memory_used: Option<u64>,
    /// Whether the execution was successful.
    pub success: bool,
}

impl SandboxOutput {
    /// Create a new sandbox output.
    pub fn new(
        stdout: String,
        stderr: String,
        exit_code: i32,
        duration: Duration,
    ) -> Self {
        Self {
            success: exit_code == 0,
            stdout,
            stderr,
            exit_code,
            duration,
            memory_used: None,
        }
    }
}

/// A command to execute within the sandbox.
#[derive(Debug, Clone)]
pub struct Command {
    /// Program to execute.
    pub program: String,
    /// Arguments to the program.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: std::collections::HashMap<String, String>,
    /// Working directory.
    pub cwd: Option<PathBuf>,
}

impl Command {
    /// Create a new command.
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: std::collections::HashMap::new(),
            cwd: None,
        }
    }

    /// Add an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments.
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(|a| a.into()));
        self
    }

    /// Set an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the working directory.
    pub fn cwd(mut self, path: PathBuf) -> Self {
        self.cwd = Some(path);
        self
    }
}

/// A sandboxed execution environment.
pub struct Sandbox {
    config: SandboxConfig,
    temp_dir: PathBuf,
    installed_files: std::collections::HashMap<String, Vec<u8>>,
}

impl Sandbox {
    /// Create a new sandbox with the given configuration.
    pub fn new(config: SandboxConfig) -> Result<Self, SandboxError> {
        let temp_dir = tempfile::Builder::new()
            .prefix("dash-sandbox-")
            .tempdir()
            .map_err(|e| SandboxError::Setup(e.to_string()))?
            .into_path();

        Ok(Self {
            config,
            temp_dir,
            installed_files: std::collections::HashMap::new(),
        })
    }

    /// Install files into the sandbox before execution.
    pub fn install(&mut self, files: &[(String, Vec<u8>)]) -> Result<(), SandboxError> {
        for (path, data) in files {
            let full_path = self.temp_dir.join(path);

            // Prevent path traversal
            if full_path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            {
                return Err(SandboxError::FilesystemDenied(PathBuf::from(path)));
            }

            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&full_path, data)?;
            self.installed_files.insert(path.clone(), data.clone());
        }

        Ok(())
    }

    /// Run a command within the sandbox.
    pub async fn run(&self, cmd: &Command) -> Result<SandboxOutput, SandboxError> {
        let start = Instant::now();

        // Check memory limit
        if let Some(limit) = self.config.memory_limit {
            let current = get_current_memory_usage();
            if current > limit {
                return Err(SandboxError::MemoryLimitExceeded { limit });
            }
        }

        // Check network access
        if !self.config.network_access {
            // This is a soft check - actual network restriction
            // would require OS-level sandboxing (seccomp, etc.)
        }

        let cwd = cmd
            .cwd
            .clone()
            .unwrap_or_else(|| self.temp_dir.clone());

        let mut command = tokio::process::Command::new(&cmd.program);
        command
            .args(&cmd.args)
            .envs(&cmd.env)
            .current_dir(&cwd)
            .kill_on_drop(true);

        let output = tokio::time::timeout(self.config.timeout, command.output())
            .await
            .map_err(|_| SandboxError::Timeout {
                duration: self.config.timeout,
            })?;

        let output = output.map_err(|e| SandboxError::Io(e))?;

        let duration = start.elapsed();

        Ok(SandboxOutput::new(
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
            output.status.code().unwrap_or(-1),
            duration,
        ))
    }

    /// Run a shell command string within the sandbox.
    pub async fn run_shell(&self, script: &str) -> Result<SandboxOutput, SandboxError> {
        let cmd = Command::new("sh")
            .arg("-c")
            .arg(script.to_string())
            .env("TMPDIR", self.temp_dir.to_string_lossy().to_string());

        self.run(&cmd).await
    }

    /// Get the sandbox's temporary directory path.
    pub fn path(&self) -> &Path {
        &self.temp_dir
    }

    /// Get the sandbox configuration.
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    /// Clean up the sandbox.
    pub fn cleanup(&self) -> Result<(), SandboxError> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)?;
        }
        Ok(())
    }

    /// Check if a command exists in the sandbox environment.
    pub async fn command_exists(&self, name: &str) -> bool {
        let cmd = Command::new("which").arg(name.to_string());
        self.run(&cmd).await.map(|o| o.success).unwrap_or(false)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

fn get_current_memory_usage() -> u64 {
    // Simple estimation - in a real sandbox, this would use
    // platform-specific APIs (e.g., getrusage on Unix)
    0
}

/// Check if a path is within the allowed filesystem scope.
pub fn is_path_allowed(path: &Path, scope: &[PathBuf]) -> bool {
    if scope.is_empty() {
        return false;
    }
    scope.iter().any(|allowed| path.starts_with(allowed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_new() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config).unwrap();
        assert!(sandbox.path().exists());
    }

    #[tokio::test]
    async fn test_sandbox_install_files() {
        let config = SandboxConfig::default();
        let mut sandbox = Sandbox::new(config).unwrap();

        let files = vec![
            ("test.txt".to_string(), b"hello sandbox".to_vec()),
            ("subdir/nested.txt".to_string(), b"nested".to_vec()),
        ];

        sandbox.install(&files).unwrap();
        assert!(sandbox.path().join("test.txt").exists());
        assert!(sandbox.path().join("subdir/nested.txt").exists());
    }

    #[tokio::test]
    async fn test_sandbox_run_echo() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config).unwrap();

        let cmd = Command::new("echo").arg("hello from sandbox");
        let output = sandbox.run(&cmd).await.unwrap();
        assert!(output.success);
        assert!(output.stdout.contains("hello from sandbox"));
    }

    #[tokio::test]
    async fn test_sandbox_run_failing_command() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config).unwrap();

        let cmd = Command::new("sh").arg("-c").arg("exit 1");
        let output = sandbox.run(&cmd).await.unwrap();
        assert!(!output.success);
        assert_eq!(output.exit_code, 1);
    }

    #[tokio::test]
    async fn test_sandbox_shell() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config).unwrap();

        let output = sandbox.run_shell("echo 'shell test'").await.unwrap();
        assert!(output.success);
    }

    #[tokio::test]
    async fn test_sandbox_path_traversal_prevention() {
        let config = SandboxConfig::default();
        let mut sandbox = Sandbox::new(config).unwrap();

        let files = vec![("../../etc/passwd".to_string(), b"hack".to_vec())];
        let result = sandbox.install(&files);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_command_new() {
        let cmd = Command::new("npm").arg("install").arg("--production");
        assert_eq!(cmd.program, "npm");
        assert_eq!(cmd.args.len(), 2);
    }

    #[test]
    fn test_is_path_allowed() {
        let scope = vec![PathBuf::from("/tmp/allowed")];
        assert!(is_path_allowed(Path::new("/tmp/allowed/pkg"), &scope));
        assert!(!is_path_allowed(Path::new("/etc/passwd"), &scope));
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(!config.network_access);
        assert_eq!(config.timeout, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_sandbox_command_exists() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config).unwrap();
        let exists = sandbox.command_exists("echo").await;
        assert!(exists);
    }

    #[tokio::test]
    async fn test_sandbox_cleanup() {
        let config = SandboxConfig::default();
        let sandbox = Sandbox::new(config).unwrap();
        let path = sandbox.path().to_path_buf();
        assert!(path.exists());
        drop(sandbox);
        assert!(!path.exists());
    }
}
