//! `bnpm remove` command

use crate::commands::RemoveArgs;
use crate::config::{CliConfig, ExitCode};
use crate::output::*;
use anyhow::{Context, Result};
use bnpm_cache::{CacheConfig, create_cache};
use bnpm_downloader::Downloader;
use bnpm_downloader::DownloaderConfig;
use bnpm_installer::{InstallStrategy, Installer, InstallerConfig};
use bnpm_linker::Linker;
use bnpm_linker::LinkerConfig;
use bnpm_lockfile::Lockfile;
use bnpm_manifest::{
    Manifest, PackageManifest, find_manifest,
    schema::Manifest as SchemaManifest,
};
use bnpm_resolver::RegistryConfig;
use bnpm_resolver::ResolveOptions;
use bnpm_resolver::Resolver;
use bnpm_store::CasStore;
use serde_json;
use std::sync::Arc;
use tokio::fs;

pub async fn run(args: &RemoveArgs, config: &CliConfig) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;

    // Find manifest
    let manifest_path = find_manifest(&cwd)
        .context("no manifest found")?;

    // Load manifest
    let mut manifest: PackageManifest = {
        let m = Manifest::load(&manifest_path).context("failed to load manifest")?;
        m.parsed
    };

    // Remove packages from all dependency sections
    let mut removed_packages = Vec::new();
    for name in &args.packages {
        if manifest.dependencies.remove(name).is_some() {
            removed_packages.push(name.clone());
        }
        if manifest.dev_dependencies.remove(name).is_some() {
            removed_packages.push(name.clone());
        }
        if manifest.peer_dependencies.remove(name).is_some() {
            removed_packages.push(name.clone());
        }
        if manifest.optional_dependencies.remove(name).is_some() {
            removed_packages.push(name.clone());
        }
    }

    if removed_packages.is_empty() {
        if config.json_output {
            print_json(&serde_json::json!({
                "removed": []
            }));
        } else {
            print_warning("No packages found to remove");
        }
        return Ok(ExitCode::Success);
    }

    // Write updated manifest
    if manifest_path.extension().and_then(|s| s.to_str()) == Some("toml") {
        let json_str = serde_json::to_string_pretty(&manifest)?;
        let schema_manifest = SchemaManifest::from_json_str(&json_str)?;
        let toml_str = schema_manifest.to_toml_string()?;
        fs::write(&manifest_path, toml_str).await?;
    } else {
        let json_str = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, json_str).await?;
    }

    // Find lockfile
    let lockfile_path = bnpm_lockfile::find_lockfile(&cwd).unwrap_or_else(|| cwd.join("bnpm.lock"));

    // Load old lockfile
    let old_lockfile = if lockfile_path.exists() {
        Lockfile::load(&lockfile_path).context("failed to load lockfile")?
    } else {
        Lockfile::with_root(format!("{}@{}", manifest.name, manifest.version))
    };

    // Re-resolve
    let resolver = create_resolver(&manifest, config)?;
    let resolve_options = ResolveOptions::new(manifest.clone())
        .with_cache(create_cache(&CacheConfig::default()))
        .offline(config.offline)
        .max_parallel(10);
    let resolved_graph = resolver
        .resolve(resolve_options)
        .await
        .context("failed to resolve dependencies")?;

    let new_lockfile =
        Resolver::to_lockfile(&resolved_graph).context("failed to create lockfile")?;

    // Write new lockfile
    if !args.no_install {
        new_lockfile.write_to_file(&lockfile_path)?;
    }

    // Install unless --no-install
    if !args.no_install {
        let installer_config = InstallerConfig {
            target_dir: cwd.join("node_modules"),
            store_dir: config.store_dir(),
            bin_dir: cwd.join("node_modules/.bin"),
            strategy: InstallStrategy::Copy,
            verify_integrity: true,
            verify_signatures: true,
            use_cache: true,
            run_scripts: true,
            concurrency: 4,
        };

        let cache = create_cache(&CacheConfig::default());
        let downloader = Downloader::new(
            reqwest::Client::new(),
            cache.clone(),
            DownloaderConfig {
                concurrency: installer_config.concurrency,
                verify_integrity: installer_config.verify_integrity,
                use_cache: installer_config.use_cache,
                registry_url: config.registry_url(),
                timeout: std::time::Duration::from_secs(120),
                resume: false,
            },
        );
        let store = Arc::new(CasStore::new(config.store_dir()).await?);
        let linker = Linker::new(
            store.clone() as Arc<dyn bnpm_store::StoreBackend>,
            LinkerConfig { bin_dir: installer_config.bin_dir.clone(), ..Default::default() },
        );
        let resolver2 = create_resolver(&manifest, config)?;
        let installer =
            Installer::new(resolver2, downloader, linker, store, installer_config, cache);

        let summary = installer.install_from_lockfile(&new_lockfile).await?;

        if !config.json_output && !config.quiet {
            print_success(&format!(
                "Removed {} packages, installed {}",
                removed_packages.len(),
                summary.installed.len()
            ));
        }
    }

    // Output
    if config.json_output {
        let removed_info: Vec<_> = args
            .packages
            .iter()
            .filter_map(|name| {
                old_lockfile.get(name).map(|p| {
                    serde_json::json!({
                        "name": p.name,
                        "version": p.version
                    })
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "removed": removed_info
        }));
    } else if !config.quiet {
        for name in &removed_packages {
            if let Some(entry) = old_lockfile.get(name) {
                print_success(&format!("Removed {}@{}", entry.name, entry.version));
            } else {
                print_success(&format!("Removed {}", name));
            }
        }
    }

    Ok(ExitCode::Success)
}

fn create_resolver(manifest: &PackageManifest, config: &CliConfig) -> Result<Resolver> {
    let cache = create_cache(&CacheConfig::default());
    let registry_config = RegistryConfig {
        url: config.registry_url(),
        token: None,
        priority: 0,
        timeout_secs: 30,
        verify_ssl: true,
    };
    let options = ResolveOptions::new(manifest.clone())
        .with_registry(registry_config)
        .with_cache(cache)
        .offline(config.offline)
        .max_parallel(10);
    Ok(Resolver::new(options))
}
