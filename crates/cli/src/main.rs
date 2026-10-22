//! Bnpm package manager - main binary entrypoint.
//!
//! Parses command-line arguments, sets up logging and telemetry,
//! and dispatches to the appropriate command handler.

use clap::{Parser, Subcommand};

/// Bnpm - The JavaScript package manager.
#[derive(Parser)]
#[command(name = "bnpm")]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Increase verbosity (can be specified multiple times: -v, -vv, -vvv)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, global = true)]
    verbosity: u8,

    /// Path to config file
    #[arg(short = 'c', long = "config", global = true)]
    config: Option<std::path::PathBuf>,

    /// Dry run (don't actually change anything)
    #[arg(long = "dry-run", global = true, default_value_t = false)]
    dry_run: bool,

    /// Disable colored output
    #[arg(long = "no-color", global = true, default_value_t = false)]
    no_color: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Install packages
    Install {
        /// Package names to install
        packages: Vec<String>,

        /// Only install production dependencies
        #[arg(long, default_value_t = false)]
        production: bool,

        /// Save to manifest
        #[arg(short, long, default_value_t = false)]
        save: bool,
    },

    /// Remove packages
    Remove {
        /// Package names to remove
        packages: Vec<String>,
    },

    /// Publish a package
    Publish {
        /// Path to the package directory
        path: Option<std::path::PathBuf>,

        /// Tag for the published version
        #[arg(short, long)]
        tag: Option<String>,

        /// Access level (public, restricted)
        #[arg(short, long)]
        access: Option<String>,
    },

    /// Search for packages
    Search {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long)]
        limit: Option<u32>,
    },

    /// Resolve dependencies
    Resolve {
        /// Path to the package
        path: Option<std::path::PathBuf>,

        /// Output as JSON
        #[arg(short, long, default_value_t = false)]
        json: bool,
    },

    /// Run a script from the manifest
    Run {
        /// Script name to run
        script: String,

        /// Arguments to pass to the script
        args: Vec<String>,
    },

    /// Initialize a new package
    Init {
        /// Path to initialize in
        path: Option<std::path::PathBuf>,

        /// Package name
        #[arg(short, long)]
        name: Option<String>,

        /// Package version
        #[arg(short, long)]
        version: Option<String>,

        /// Skip prompts
        #[arg(short, long, default_value_t = false)]
        yes: bool,
    },

    /// Execute a plugin command
    Plugin {
        /// Plugin name
        name: String,

        /// Plugin arguments
        args: Vec<String>,
    },

    /// Manage the cache
    Cache {
        /// Cache action
        #[command(subcommand)]
        action: CacheCommands,
    },

    /// Workspace operations
    Workspace {
        #[command(subcommand)]
        action: WorkspaceCommands,
    },
}

#[derive(clap::Subcommand)]
enum CacheCommands {
    /// Clear the cache
    Clear,
    /// Show cache statistics
    Stats,
    /// Prune expired cache entries
    Prune,
}

#[derive(clap::Subcommand)]
enum WorkspaceCommands {
    /// List workspace members
    List,
    /// Show workspace or member info
    Info {
        /// Member name (omit for workspace info)
        member: Option<String>,
    },
    /// Show the workspace dependency graph
    Graph,
}

fn main() {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = match cli.verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let log_config = bnpm_logger::LogConfig {
        level: log_level.to_string(),
        format: bnpm_logger::LogFormat::Text,
        output: bnpm_logger::LogOutput::Stdout,
        include_spans: true,
    };

    if let Err(e) = bnpm_logger::init(&log_config) {
        eprintln!("Warning: failed to initialize logger: {e}");
    }

    // Initialize telemetry
    let telemetry_config = bnpm_telemetry::TelemetryConfig {
        service_name: "bnpm-cli".to_string(),
        log_level: log_level.to_string(),
        ..Default::default()
    };

    if let Err(e) = bnpm_telemetry::init_tracing(&telemetry_config) {
        tracing::warn!("Failed to initialize telemetry: {e}");
    }

    // Build the command from CLI args
    let command = match cli.command {
        Some(Commands::Install { packages, production, save }) => {
            bnpm_cli::Command::Install { packages, production, save }
        }
        Some(Commands::Remove { packages }) => {
            bnpm_cli::Command::Remove { packages }
        }
        Some(Commands::Publish { path, tag, access }) => {
            bnpm_cli::Command::Publish { path, tag, access }
        }
        Some(Commands::Search { query, limit }) => {
            bnpm_cli::Command::Search { query, limit }
        }
        Some(Commands::Resolve { path, json }) => {
            bnpm_cli::Command::Resolve { path, json }
        }
        Some(Commands::Run { script, args }) => {
            bnpm_cli::Command::Run { script, args }
        }
        Some(Commands::Init { path, name, version, yes }) => {
            bnpm_cli::Command::Init { path, name, version, yes }
        }
        Some(Commands::Plugin { name, args }) => {
            bnpm_cli::Command::Plugin { name, args }
        }
        Some(Commands::Cache { action }) => {
            let cache_action = match action {
                CacheCommands::Clear => bnpm_cli::CacheAction::Clear,
                CacheCommands::Stats => bnpm_cli::CacheAction::Stats,
                CacheCommands::Prune => bnpm_cli::CacheAction::Prune,
            };
            bnpm_cli::Command::Cache { action: cache_action }
        }
        Some(Commands::Workspace { action }) => {
            let workspace_action = match action {
                WorkspaceCommands::List => bnpm_cli::WorkspaceAction::List,
                WorkspaceCommands::Info { member } => bnpm_cli::WorkspaceAction::Info { member },
                WorkspaceCommands::Graph => bnpm_cli::WorkspaceAction::Graph,
            };
            bnpm_cli::Command::Workspace { action: workspace_action }
        }
        None => {
            // No subcommand - show help
            bnpm_cli::Command::Help { command: None }
        }
    };

    // Load config and set verbosity
    let mut cli_config = match bnpm_cli::CliConfig::load() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load config: {e}");
            std::process::exit(bnpm_cli::ExitCode::Error.code());
        }
    };

    cli_config.set_verbosity(cli.verbosity);
    cli_config.dry_run = cli.dry_run;
    cli_config.color = !cli.no_color;
    cli_config.config_file = cli.config;

    // Execute the command
    let exit_code = match bnpm_cli::run(&command) {
        Ok(code) => code,
        Err(e) => {
            tracing::error!("{e}");
            bnpm_cli::ExitCode::Error
        }
    };

    // Shutdown telemetry gracefully
    let _ = bnpm_telemetry::shutdown_tracing();

    std::process::exit(exit_code.code());
}
