//! `bnpm pack` command

use crate::commands::PackArgs;
use crate::config::{CliConfig, ExitCode};
use anyhow::Result;

pub async fn run(_args: &PackArgs, _config: &CliConfig) -> Result<ExitCode> {
    Ok(ExitCode::Success)
}
