//! `bnpm run` command

use crate::commands::RunArgs;
use crate::config::{CliConfig, ExitCode};
use anyhow::Result;

pub async fn run(_args: &RunArgs, _config: &CliConfig) -> Result<ExitCode> {
    Ok(ExitCode::Success)
}
