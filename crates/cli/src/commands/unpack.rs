//! `dash unpack` command

use crate::commands::UnpackArgs;
use crate::config::{CliConfig, ExitCode};
use anyhow::Result;

pub async fn run(_args: &UnpackArgs, _config: &CliConfig) -> Result<ExitCode> {
    Ok(ExitCode::Success)
}
