//! CLI command implementations.

pub mod add;
pub mod agent;
pub mod audit;
pub mod completions;
pub mod doctor;
pub mod help;
pub mod init;
pub mod install;
pub mod pack;
pub mod publish;
pub mod remove;
pub mod run;
pub mod unpack;
pub mod up;
pub mod verify;
pub mod version;
pub mod why;

/// Command enum for dispatch
#[derive(Debug, Clone)]
pub enum Command {
    Init(InitArgs),
    Add(AddArgs),
    Remove(RemoveArgs),
    Install(InstallArgs),
    Up(UpArgs),
    Run(RunArgs),
    Pack(PackArgs),
    Publish(PublishArgs),
    Unpack(UnpackArgs),
    Audit(AuditArgs),
    Verify(VerifyArgs),
    Doctor(DoctorArgs),
    Completions { shell: clap_complete::Shell },
    Version,
    Help { command: Option<String> },
    Why(WhyArgs),
    Agent(AgentArgs),
}

#[derive(Debug, Clone)]
pub struct InitArgs {
    pub path: std::path::PathBuf,
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

#[derive(Debug, Clone)]
pub struct AddArgs {
    pub spec: String,
    pub dev: bool,
    pub peer: bool,
    pub optional: bool,
    pub exact: bool,
    pub why: bool,
    pub no_install: bool,
}

#[derive(Debug, Clone)]
pub struct RemoveArgs {
    pub packages: Vec<String>,
    pub no_install: bool,
}

#[derive(Debug, Clone)]
pub struct InstallArgs {
    pub packages: Option<Vec<String>>,
    pub production: bool,
    pub offline: bool,
    pub frozen_lock: bool,
    pub no_dev: bool,
    pub vendor: bool,
}

#[derive(Debug, Clone)]
pub struct UpArgs {
    pub packages: Option<Vec<String>>,
    pub range: Option<String>,
    pub why: bool,
    pub capability_diff: bool,
    pub api_diff: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct RunArgs {
    pub script: String,
    pub args: Vec<String>,
    pub list: bool,
    pub affected: bool,
}

#[derive(Debug, Clone)]
pub struct PackArgs {
    pub path: std::path::PathBuf,
    pub output: Option<std::path::PathBuf>,
    pub ignore: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UnpackArgs {
    pub capsule: std::path::PathBuf,
    pub output: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct PublishArgs {
    pub path: std::path::PathBuf,
    pub registry: Option<String>,
    pub dual: bool,
    pub npm_tag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AuditArgs {
    pub json: bool,
    pub reputation: bool,
    pub governance: bool,
}

#[derive(Debug, Clone)]
pub struct VerifyArgs {
    pub checkpoint: bool,
    pub strict: bool,
}

#[derive(Debug, Clone)]
pub struct WhyArgs {
    pub package: String,
    pub provenance: bool,
    pub graph: bool,
    pub capabilities: bool,
}

#[derive(Debug, Clone)]
pub struct DoctorArgs {
    pub verbose: bool,
}

#[derive(Debug, Clone)]
pub struct AgentArgs {
    pub subcommand: AgentSubcommand,
}

#[derive(Debug, Clone)]
pub enum AgentSubcommand {
    Select { query: String },
    Explain { package: String },
    PlanUpgrade { goal: Option<String> },
    Audit,
    Compose { tool_spec: Option<String> },
}
