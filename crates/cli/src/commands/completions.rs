//! `dash completions` command - generate shell completions

use crate::config::ExitCode;
use anyhow::Result;
use clap::Command;
use clap_complete::{Shell, generate};
use std::io;

pub fn run(shell: Shell, _config: &crate::config::CliConfig) -> Result<ExitCode> {
    let mut cmd = Command::new("dash")
        .subcommand(Command::new("init"))
        .subcommand(Command::new("add"))
        .subcommand(Command::new("remove"))
        .subcommand(Command::new("install").alias("i"))
        .subcommand(Command::new("up"))
        .subcommand(Command::new("run"))
        .subcommand(Command::new("pack"))
        .subcommand(Command::new("unpack"))
        .subcommand(Command::new("verify"))
        .subcommand(Command::new("doctor"))
        .subcommand(Command::new("completions"))
        .subcommand(Command::new("version"))
        .subcommand(Command::new("help"));
    generate(shell, &mut cmd, "dash", &mut io::stdout());
    Ok(ExitCode::Success)
}
