//! `dash init` command - scaffold a new package

use crate::commands::InitArgs;
use crate::config::{CliConfig, ExitCode};
use crate::output::*;
use anyhow::Result;
use dash_manifest::schema::Manifest;

pub async fn run(args: &InitArgs, _config: &CliConfig) -> Result<ExitCode> {
    let path = if args.path.as_os_str().is_empty() {
        std::path::PathBuf::from(".")
    } else {
        args.path.clone()
    };
    let manifest_path = path.join("dash.toml");

    let manifest = Manifest {
        package: Some(dash_manifest::schema::PackageIdentity {
            name: args.name.clone().unwrap_or_else(|| "my-package".to_string()),
            version: args.version.clone().unwrap_or_else(|| "0.0.1".parse().unwrap()),
            description: Some("A Dash package".to_string()),
            license: Some(args.license.clone()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let toml = toml::to_string_pretty(&manifest)?;
    std::fs::write(&manifest_path, toml)?;
    print_success(&format!("Created {}", manifest_path.display()));
    Ok(ExitCode::Success)
}
