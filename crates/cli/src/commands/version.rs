//! `dash version` command

use crate::config::{CliConfig, ExitCode};
use anyhow::Result;

pub fn run(_config: &CliConfig) -> Result<ExitCode> {
    println!("dash {}", env!("CARGO_PKG_VERSION"));
    Ok(ExitCode::Success)
}
