//! `bnpm doctor` command

use crate::commands::DoctorArgs;
use crate::config::{CliConfig, ExitCode};
use anyhow::Result;

pub async fn run(_args: &DoctorArgs, _config: &CliConfig) -> Result<ExitCode> {
    Ok(ExitCode::Success)
}
