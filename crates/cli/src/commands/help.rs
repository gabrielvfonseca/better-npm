//! `bnpm help` command

use crate::config::{CliConfig, ExitCode};
use anyhow::Result;

pub fn run(_command: Option<String>, _config: &CliConfig) -> Result<ExitCode> {
    println!("Bnpm - The JavaScript package manager");
    println!("Usage: bnpm <command> [options]");
    println!();
    println!("Commands:");
    println!("  init      Initialize a new package");
    println!("  add       Add a dependency");
    println!("  remove    Remove a dependency");
    println!("  install   Install dependencies");
    println!("  up        Upgrade dependencies");
    println!("  run       Run a script");
    println!("  pack      Create a capsule");
    println!("  unpack    Extract a capsule");
    println!("  verify    Verify capsule integrity");
    println!("  doctor    Diagnose environment");
    println!("  version   Show version");
    println!("  help      Show this help");
    Ok(ExitCode::Success)
}
