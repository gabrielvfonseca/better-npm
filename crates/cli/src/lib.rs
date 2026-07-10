//! Dash package manager CLI - main binary entrypoint.
//!
//! This crate provides the CLI entrypoint for Dash, including
//! subcommand dispatch, configuration loading, and exit code handling.

use std::path::PathBuf;

/// Re-export commonly used types for convenience.
pub use dash_config::Config;
pub use dash_package::{Package, PackageId, PackageName};
pub use dash_resolver::Resolver;
pub use dash_installer::Installer;

/// Error type for CLI operations.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("config error: {0}")]
    Config(#[from] dash_config::ConfigError),
    #[error("install error: {0}")]
    Install(#[from] dash_installer::InstallError),
    #[error("resolve error: {0}")]
    Resolve(#[from] dash_resolver::ResolveError),
    #[error("manifest error: {0}")]
    Manifest(#[from] dash_manifest::ManifestError),
    #[error("lockfile error: {0}")]
    Lockfile(#[from] dash_lockfile::LockfileError),
    #[error("network error: {0}")]
    Network(#[from] dash_network::NetworkError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("logger error: {0}")]
    Logger(#[from] dash_logger::LogError),
    #[error("telemetry error: {0}")]
    Telemetry(#[from] dash_telemetry::TelemetryError),
    #[error("diagnostic error: {0}")]
    Diagnostic(#[from] dash_diagnostics::DiagnosticError),
    #[error("plugin error: {0}")]
    Plugin(#[from] dash_plugin::PluginError),
    #[error("sandbox error: {0}")]
    Sandbox(#[from] dash_sandbox::SandboxError),
    #[error("cache error: {0}")]
    Cache(#[from] dash_cache::CacheError),
    #[error("store error: {0}")]
    Store(#[from] dash_store::StoreError),
    #[error("concurrency error: {0}")]
    Concurrency(#[from] dash_concurrency::ConcurrencyError),
    #[error("package error: {0}")]
    Package(#[from] dash_package::PackageError),
    #[error("publisher error: {0}")]
    Publisher(#[from] dash_publisher::PublishError),
    #[error("registry error: {0}")]
    Registry(#[from] dash_registry_client::RegistryError),
    #[error("runtime error: {0}")]
    Runtime(#[from] dash_runtime::RuntimeError),
    #[error("workspace error: {0}")]
    Workspace(#[from] dash_workspace::WorkspaceError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

/// Exit code for the CLI binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Successful completion.
    Success = 0,
    /// General error.
    Error = 1,
    /// Invalid usage / bad arguments.
    Usage = 2,
    /// Permission denied.
    PermissionDenied = 3,
    /// Network error.
    NetworkError = 4,
    /// Package not found.
    NotFound = 5,
    /// Operation cancelled.
    Cancelled = 130,
}

impl ExitCode {
    /// Return the exit code as an i32.
    pub fn code(&self) -> i32 {
        *self as i32
    }
}

/// CLI configuration loaded from files and command-line arguments.
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Global dash configuration.
    pub config: dash_config::Config,
    /// Verbosity level (0 = default, 1 = verbose, 2 = debug).
    pub verbosity: u8,
    /// Whether to run in dry-run mode.
    pub dry_run: bool,
    /// Whether to use colors in output.
    pub color: bool,
    /// Custom config file path.
    pub config_file: Option<PathBuf>,
    /// Working directory.
    pub cwd: PathBuf,
}

impl CliConfig {
    /// Load the CLI configuration.
    pub fn load() -> Result<Self, CliError> {
        let mut config = dash_config::load_config()?;

        let verbosity: u8 = config.get_or("cli.verbosity", 0u8);
        let dry_run: bool = config.get_or("cli.dry_run", false);
        let color: bool = config.get_or("cli.color", true);

        Ok(Self {
            config,
            verbosity,
            dry_run,
            color,
            config_file: None,
            cwd: std::env::current_dir()?,
        })
    }

    /// Create a new CLI config with defaults.
    pub fn new() -> Self {
        Self {
            config: dash_config::Config::empty(),
            verbosity: 0,
            dry_run: false,
            color: true,
            config_file: None,
            cwd: std::env::current_dir().unwrap_or_default(),
        }
    }

    /// Set verbosity from a count (e.g., -v, -vv, -vvv).
    pub fn set_verbosity(&mut self, count: u8) {
        self.verbosity = count;
        // Update the log level based on verbosity
        let level = match count {
            0 => "info",
            1 => "debug",
            _ => "trace",
        };
        self.config.set("log.level", level);
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// The top-level command enum for the CLI.
#[derive(Debug, Clone)]
pub enum Command {
    /// Install packages.
    Install {
        packages: Vec<String>,
        production: bool,
        save: bool,
    },
    /// Remove packages.
    Remove {
        packages: Vec<String>,
    },
    /// Publish a package.
    Publish {
        path: Option<PathBuf>,
        tag: Option<String>,
        access: Option<String>,
    },
    /// Search for packages.
    Search {
        query: String,
        limit: Option<u32>,
    },
    /// Resolve dependencies (show resolution).
    Resolve {
        path: Option<PathBuf>,
        json: bool,
    },
    /// Run a script from the manifest.
    Run {
        script: String,
        args: Vec<String>,
    },
    /// Initialize a new package.
    Init {
        path: Option<PathBuf>,
        name: Option<String>,
        version: Option<String>,
        yes: bool,
    },
    /// Display version information.
    Version,
    /// Display help information.
    Help {
        command: Option<String>,
    },
    /// Execute a plugin command.
    Plugin {
        name: String,
        args: Vec<String>,
    },
    /// Manage the cache.
    Cache {
        action: CacheAction,
    },
    /// Workspace operations.
    Workspace {
        action: WorkspaceAction,
    },
}

/// Cache subcommands.
#[derive(Debug, Clone)]
pub enum CacheAction {
    Clear,
    Stats,
    Prune,
}

/// Workspace subcommands.
#[derive(Debug, Clone)]
pub enum WorkspaceAction {
    List,
    Info { member: Option<String> },
    Graph,
}

/// Run the CLI with the given command, returning an exit code.
pub fn run(command: &Command) -> Result<ExitCode, CliError> {
    let config = CliConfig::load()?;

    match command {
        Command::Install { packages, production, save } => {
            run_install(&config, packages, *production, *save)
        }
        Command::Remove { packages } => run_remove(&config, packages),
        Command::Publish { path, tag, access } => {
            run_publish(&config, path.as_deref(), tag.as_deref(), access.as_deref())
        }
        Command::Search { query, limit } => run_search(&config, query, *limit),
        Command::Resolve { path, json } => run_resolve(&config, path.as_deref(), *json),
        Command::Run { script, args } => run_script(&config, script, args),
        Command::Init { path, name, version, yes } => {
            run_init(path.as_deref(), name.as_deref(), version.as_deref(), *yes)
        }
        Command::Version => run_version(),
        Command::Help { command } => run_help(command.as_deref()),
        Command::Plugin { name, args } => run_plugin(&config, name, args),
        Command::Cache { action } => run_cache(&config, action),
        Command::Workspace { action } => run_workspace(&config, action),
    }
}

fn run_install(config: &CliConfig, packages: &[String], production: bool, _save: bool) -> Result<ExitCode, CliError> {
    let installer = dash_installer::create_installer(&config.config)?;

    let rt = dash_runtime::init_default_runtime()?;
    rt.block_on(async {
        let pkg_names: Vec<dash_package::PackageName> = packages
            .iter()
            .map(|p| dash_package::PackageName::new(p).unwrap_or_else(|_| dash_package::PackageName::new_unchecked(p)))
            .collect();

        if production {
            for name in &pkg_names {
                let summary = installer.install(name).await?;
                tracing::info!("Installed {} packages in {:?}", summary.package_count, summary.duration);
            }
        } else {
            let summaries = installer.install_many(&pkg_names).await?;
            for s in &summaries {
                tracing::info!("Installed {} package(s) in {:?}", s.package_count, s.duration);
            }
        }

        Ok::<_, CliError>(())
    })?;

    tracing::info!("Install complete");
    Ok(ExitCode::Success)
}

fn run_remove(config: &CliConfig, packages: &[String]) -> Result<ExitCode, CliError> {
    let installer = dash_installer::create_installer(&config.config)?;

    let rt = dash_runtime::init_default_runtime()?;
    rt.block_on(async {
        for pkg in packages {
            let name = dash_package::PackageName::new(pkg)
                .map_err(|e| CliError::Package(e))?;
            installer.remove(&name).await?;
            tracing::info!("Removed {pkg}");
        }
        Ok::<_, CliError>(())
    })?;

    Ok(ExitCode::Success)
}

fn run_publish(
    config: &CliConfig,
    path: Option<&std::path::Path>,
    _tag: Option<&str>,
    _access: Option<&str>,
) -> Result<ExitCode, CliError> {
    let publisher = dash_publisher::create_publisher_from_config(&config.config)?;

    let path = path.map_or_else(|| config.cwd.clone(), |p| p.to_path_buf());
    let manifest_path = dash_manifest::find_manifest(&path)
        .ok_or_else(|| CliError::Other("manifest not found".to_string()))?;

    let manifest = dash_manifest::Manifest::load(&manifest_path)?;
    let name = dash_package::PackageName::new(manifest.name())?;
    let version: semver::Version = manifest.version().parse()
        .map_err(|e| CliError::Other(format!("invalid version: {e}")))?;

    let pkg = dash_package::Package::new(name, version);

    let rt = dash_runtime::init_default_runtime()?;
    let result = rt.block_on(publisher.publish(&pkg))?;

    tracing::info!("Published {}@{} (revision: {:?})", result.name, result.version, result.revision);
    Ok(ExitCode::Success)
}

fn run_search(config: &CliConfig, query: &str, limit: Option<u32>) -> Result<ExitCode, CliError> {
    let client = dash_registry_client::RegistryClient::from_config(&config.config)?;

    let rt = dash_runtime::init_default_runtime()?;
    let results = rt.block_on(client.search(query, limit, None))?;

    for result in &results.objects {
        println!("{}@{}", result.package.name, result.package.version);
        if let Some(ref desc) = result.package.description {
            println!("  {desc}");
        }
    }

    tracing::info!("Found {} results", results.total);
    Ok(ExitCode::Success)
}

fn run_resolve(config: &CliConfig, _path: Option<&std::path::Path>, json: bool) -> Result<ExitCode, CliError> {
    let resolver = dash_resolver::create_resolver(&config.config);

    // Load manifest
    let mut deps = dash_resolver::DependencySet::new();
    deps.add(dash_resolver::Dependency::new("example-pkg", "*"));

    let graph = resolver.resolve(&deps)?;

    if json {
        let output = serde_json::to_string_pretty(&graph.nodes)
            .map_err(|e| CliError::Other(e.to_string()))?;
        println!("{output}");
    } else {
        for (name, version) in &graph.nodes {
            println!("{name}@{version}");
        }
    }

    Ok(ExitCode::Success)
}

fn run_script(_config: &CliConfig, script: &str, _args: &[String]) -> Result<ExitCode, CliError> {
    tracing::info!("Running script: {script}");
    // TODO: Look up script in manifest and execute
    Err(CliError::Other(format!("script '{}' not yet implemented", script)))
}

fn run_init(
    path: Option<&std::path::Path>,
    name: Option<&str>,
    version: Option<&str>,
    _yes: bool,
) -> Result<ExitCode, CliError> {
    let dir = path.map_or_else(|| std::env::current_dir().unwrap(), |p| p.to_path_buf());
    let name = name.unwrap_or("my-package");
    let version = version.unwrap_or("0.1.0");

    let manifest = serde_json::json!({
        "name": name,
        "version": version,
        "description": "",
        "main": "index.js",
        "scripts": {
            "test": "echo \"Error: no test specified\" && exit 1"
        },
        "keywords": [],
        "author": "",
        "license": "MIT"
    });

    let manifest_path = dir.join("package.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;

    tracing::info!("Initialized package at {}", manifest_path.display());
    Ok(ExitCode::Success)
}

fn run_version() -> Result<ExitCode, CliError> {
    println!("Dash v{}", env!("CARGO_PKG_VERSION"));
    Ok(ExitCode::Success)
}

fn run_help(command: Option<&str>) -> Result<ExitCode, CliError> {
    match command {
        Some("install") => println!("dash install [package...]  Install packages"),
        Some("remove") => println!("dash remove [package...]   Remove packages"),
        Some("publish") => println!("dash publish [path]        Publish a package"),
        Some("search") => println!("dash search <query>        Search for packages"),
        Some("init") => println!("dash init [path]            Initialize a new package"),
        Some("run") => println!("dash run <script>           Run a script"),
        Some("cache") => println!("dash cache <clear|stats|prune>  Manage cache"),
        Some("workspace") => println!("dash workspace <list|info|graph>  Workspace operations"),
        None => {
            println!("Dash - The JavaScript package manager");
            println!();
            println!("Usage: dash <command> [options]");
            println!();
            println!("Commands:");
            println!("  install     Install packages");
            println!("  remove      Remove packages");
            println!("  publish     Publish a package");
            println!("  search      Search for packages");
            println!("  resolve     Resolve dependencies");
            println!("  run         Run a script");
            println!("  init        Initialize a new package");
            println!("  cache       Manage cache");
            println!("  workspace   Workspace operations");
            println!("  help        Show this help");
            println!("  version     Show version");
            println!();
            println!("Use 'dash help <command>' for more details.");
        }
        Some(other) => println!("Unknown command: {other}"),
    }
    Ok(ExitCode::Success)
}

fn run_plugin(config: &CliConfig, name: &str, args: &[String]) -> Result<ExitCode, CliError> {
    let mut plugin_manager = dash_plugin::PluginManager::new();

    // Try to load plugin from standard directories
    let plugin_dirs = vec![
        config.cwd.join(".dash").join("plugins"),
        dirs::config_dir()
            .map(|p| p.join("dash").join("plugins"))
            .unwrap_or_default(),
    ];

    for dir in &plugin_dirs {
        if dir.exists() {
            dash_plugin::load_plugins_from_dir(&mut plugin_manager, dir)?;
        }
    }

    if let Some(plugin) = plugin_manager.get(name) {
        tracing::info!("Running plugin '{}' v{}", plugin.name(), plugin.version());
        plugin_manager.execute_command(name, args)?;
    } else {
        return Err(CliError::Other(format!("plugin '{name}' not found")));
    }

    Ok(ExitCode::Success)
}

fn run_cache(config: &CliConfig, action: &CacheAction) -> Result<ExitCode, CliError> {
    match action {
        CacheAction::Clear => {
            let cache = dash_cache::create_cache(&dash_cache::CacheConfig::default());
            let rt = dash_runtime::init_default_runtime()?;
            rt.block_on(async {
                cache.clear().await?;
                tracing::info!("Cache cleared");
                Ok::<_, CliError>(())
            })?;
        }
        CacheAction::Stats => {
            tracing::info!("Cache stats not yet implemented");
        }
        CacheAction::Prune => {
            tracing::info!("Cache pruning not yet implemented");
        }
    }
    Ok(ExitCode::Success)
}

fn run_workspace(config: &CliConfig, action: &WorkspaceAction) -> Result<ExitCode, CliError> {
    let workspace = dash_workspace::discover_workspace()?;

    match action {
        WorkspaceAction::List => {
            for member in workspace.packages() {
                let root_indicator = if member.is_root { " (root)" } else { "" };
                println!("{}@{}  {}{}", member.name, member.version, member.path.display(), root_indicator);
            }
        }
        WorkspaceAction::Info { member } => {
            if let Some(name) = member {
                let m = workspace
                    .member(name)
                    .ok_or_else(|| CliError::Other(format!("member '{name}' not found")))?;
                println!("Name:    {}", m.name);
                println!("Version: {}", m.version);
                println!("Path:    {}", m.path.display());
                println!("Dependencies: {}", m.manifest.dependencies.len());
            } else {
                println!("Workspace root: {}", workspace.root().display());
                println!("Members: {}", workspace.len());
                for m in workspace.packages() {
                    println!("  {}@{}", m.name, m.version);
                }
            }
        }
        WorkspaceAction::Graph => {
            let graph = workspace.dependency_graph();
            println!("{}", graph.visualize());
        }
    }

    Ok(ExitCode::Success)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_values() {
        assert_eq!(ExitCode::Success.code(), 0);
        assert_eq!(ExitCode::Error.code(), 1);
        assert_eq!(ExitCode::Usage.code(), 2);
    }

    #[test]
    fn test_cli_config_new() {
        let config = CliConfig::new();
        assert_eq!(config.verbosity, 0);
        assert!(!config.dry_run);
    }

    #[test]
    fn test_cli_config_set_verbosity() {
        let mut config = CliConfig::new();
        config.set_verbosity(2);
        assert_eq!(config.verbosity, 2);
        assert_eq!(
            config.config.get::<String>("log.level").unwrap(),
            "trace"
        );
    }

    #[test]
    fn test_run_version() {
        let result = run_version();
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_help() {
        let result = run_help(None);
        assert!(result.is_ok());

        let result = run_help(Some("install"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_init() {
        let dir = tempfile::tempdir().unwrap();
        let result = run_init(Some(&dir.path().to_path_buf()), Some("test-pkg"), Some("1.0.0"), true);
        assert!(result.is_ok());
        assert!(dir.path().join("package.json").exists());
    }

    #[test]
    fn test_cache_actions() {
        // Just verify the enum variants
        match CacheAction::Clear {
            CacheAction::Clear => {}
            _ => panic!("wrong variant"),
        }
        match CacheAction::Stats {
            CacheAction::Stats => {}
            _ => panic!("wrong variant"),
        }
        match CacheAction::Prune {
            CacheAction::Prune => {}
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_workspace_actions() {
        match WorkspaceAction::List {
            WorkspaceAction::List => {}
            _ => panic!("wrong variant"),
        }
        let action = WorkspaceAction::Info { member: None };
        match action {
            WorkspaceAction::Info { .. } => {}
            _ => panic!("wrong variant"),
        }
        match WorkspaceAction::Graph {
            WorkspaceAction::Graph => {}
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_command_variants() {
        // Verify all command variants are constructible
        let _ = Command::Install {
            packages: vec!["lodash".to_string()],
            production: false,
            save: true,
        };
        let _ = Command::Remove {
            packages: vec!["lodash".to_string()],
        };
        let _ = Command::Publish {
            path: None,
            tag: None,
            access: None,
        };
        let _ = Command::Search {
            query: "react".to_string(),
            limit: None,
        };
        let _ = Command::Version;
        let _ = Command::Help { command: None };
    }

    #[test]
    fn test_dry_run_default() {
        let config = CliConfig::new();
        assert!(!config.dry_run);
    }
}
