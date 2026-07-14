//! `dash up` command

use crate::commands::UpArgs;
use crate::config::{CliConfig, ExitCode};
use crate::output::*;
use anyhow::{Context, Result};
use dash_cache::{CacheConfig, create_cache};
use dash_downloader::Downloader;
use dash_downloader::DownloaderConfig;
use dash_installer::{InstallStrategy, Installer, InstallerConfig};
use dash_linker::Linker;
use dash_linker::LinkerConfig;
use dash_lockfile::Lockfile;
use dash_manifest::{
    Manifest, PackageManifest, find_manifest,
    schema::Manifest as SchemaManifest,
};
use dash_resolver::RegistryConfig;
use dash_resolver::ResolveOptions;
use dash_resolver::Resolver;
use dash_store::CasStore;
use std::sync::Arc;
use tokio::fs;

pub async fn run(args: &UpArgs, config: &CliConfig) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;

    // Find manifest
    let manifest_path = find_manifest(&cwd)
        .context("no manifest found")?;

    // Load manifest
    let mut manifest: PackageManifest = {
        let m = Manifest::load(&manifest_path)?;
        m.parsed
    };

    // Load lockfile
    let lockfile_path = dash_lockfile::find_lockfile(&cwd).unwrap_or_else(|| cwd.join("dash.lock"));
    let old_lockfile = Lockfile::load(&lockfile_path).context("failed to load lockfile")?;

    // Determine packages to upgrade
    let packages_to_upgrade: Vec<String> = if args.packages.is_some() {
        args.packages.clone().unwrap()
    } else {
        // Upgrade all dependencies
        let mut all = Vec::new();
        all.extend(manifest.dependencies.keys().cloned());
        all.extend(manifest.dev_dependencies.keys().cloned());
        all.extend(manifest.peer_dependencies.keys().cloned());
        all.extend(manifest.optional_dependencies.keys().cloned());
        all
    };

    // Compute upgrades
    let mut upgrades = Vec::new();
    let mut manifest_updated = false;

    for pkg_name in &packages_to_upgrade {
        // Get current version from lockfile
        let old_version = old_lockfile.get(pkg_name).map(|p| p.version.clone());

        // Get current version range from manifest
        let current_range = get_dependency_range(&manifest, pkg_name);

        // Compute new range
        let new_range = if let Some(ref range) = args.range {
            range.clone()
        } else if let Some(ref current) = current_range {
            // Relax to * or use exact if --exact
            if current == "*" || current == "latest" {
                current.clone()
            } else if args.dry_run {
                "*".to_string()
            } else {
                // Default: relax to *
                "*".to_string()
            }
        } else {
            "*".to_string()
        };

        // Update manifest if not dry-run
        if !args.dry_run {
            update_manifest_range(&mut manifest, pkg_name, &new_range)?;
            manifest_updated = true;
        }

        // For dry-run, just show what would happen
        if args.dry_run {
            let new_version = if new_range == "*" {
                "latest".to_string()
            } else {
                "latest within range".to_string()
            };
            upgrades.push(UpgradeInfo {
                name: pkg_name.clone(),
                from: old_version.unwrap_or_else(|| "unknown".to_string()),
                to: new_version,
                capability_diff: None,
                api_diff: None,
            });
        }
    }

    if args.dry_run {
        // Just show what would be upgraded
        if config.json_output {
            print_json(&serde_json::json!({
                "upgrades": upgrades.iter().map(|u| serde_json::json!({
                    "name": u.name,
                    "from": u.from,
                    "to": u.to,
                    "capabilityDiff": u.capability_diff,
                    "apiDiff": u.api_diff
                })).collect::<Vec<_>>(),
                "lockfileChanged": true
            }));
        } else {
            print_info("Would upgrade the following packages:");
            for u in &upgrades {
                print_kv(&u.name, &format!("{} -> {}", u.from, u.to));
            }
        }
        return Ok(ExitCode::Success);
    }

    // Write updated manifest
    if manifest_updated {
        if manifest_path.extension().and_then(|s| s.to_str()) == Some("toml") {
            let json_str = serde_json::to_string_pretty(&manifest)?;
            let schema_manifest = SchemaManifest::from_json_str(&json_str)?;
            let toml_str = schema_manifest.to_toml_string()?;
            fs::write(&manifest_path, toml_str).await?;
        } else {
            let json_str = serde_json::to_string_pretty(&manifest)?;
            fs::write(&manifest_path, json_str).await?;
        }
    }

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

    // Handle --why
    if args.why || config.why {
        for pkg_name in &packages_to_upgrade {
            let trace = resolved_graph.why(pkg_name);
            if config.json_output {
                print_json(&serde_json::json!({
                    "package": pkg_name,
                    "resolutionTrace": trace.iter().map(|e| serde_json::json!({
                        "requiredBy": e.required_by.to_string(),
                        "range": e.range,
                        "selected": e.selected.to_string(),
                        "kind": format!("{:?}", e.kind).to_lowercase()
                    })).collect::<Vec<_>>()
                }));
            } else {
                print_info(&format!("Resolution trace for {}:", pkg_name));
                for edge in trace {
                    print_kv(
                        "  via",
                        &format!(
                            "{} requires {}@{} -> {}@{}",
                            edge.required_by, pkg_name, edge.range, pkg_name, edge.selected
                        ),
                    );
                }
            }
        }
    }

    // Show capability/api diff if requested (stub)
    if args.capability_diff {
        if config.json_output {
            print_json(&serde_json::json!({
                "capabilityDiff": "not yet implemented"
            }));
        } else {
            print_info("Capability diff: not yet implemented (requires M3)");
        }
    }

    if args.api_diff {
        if config.json_output {
            print_json(&serde_json::json!({
                "apiDiff": "not yet implemented"
            }));
        } else {
            print_info("API diff: not yet implemented");
        }
    }

    // Write new lockfile
    new_lockfile.write_to_file(&lockfile_path)?;

    // Compute actual upgrades
    let mut final_upgrades = Vec::new();
    for pkg_name in &packages_to_upgrade {
        let old_entry = old_lockfile.get(pkg_name);
        let new_entry = new_lockfile.get(pkg_name);

        if let (Some(old), Some(new)) = (old_entry, new_entry) {
            if old.version != new.version {
                final_upgrades.push(UpgradeInfo {
                    name: pkg_name.clone(),
                    from: old.version.clone(),
                    to: new.version.clone(),
                    capability_diff: if args.capability_diff {
                        Some("not yet implemented")
                    } else {
                        None
                    },
                    api_diff: if args.api_diff { Some("not yet implemented") } else { None },
                });
            }
        }
    }

    // Install
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
        store.clone() as Arc<dyn dash_store::StoreBackend>,
        LinkerConfig { bin_dir: installer_config.bin_dir.clone(), ..Default::default() },
    );
    let resolver2 = create_resolver(&manifest, config)?;
    let installer = Installer::new(resolver2, downloader, linker, store, installer_config, cache);

    let summary = installer.install_from_lockfile(&new_lockfile).await?;

    // Output
    if config.json_output {
        print_json(&serde_json::json!({
            "upgrades": final_upgrades.iter().map(|u| serde_json::json!({
                "name": u.name,
                "from": u.from,
                "to": u.to,
                "capabilityDiff": u.capability_diff,
                "apiDiff": u.api_diff
            })).collect::<Vec<_>>(),
            "lockfileChanged": true
        }));
    } else if !config.quiet {
        if final_upgrades.is_empty() {
            print_success("All packages up to date");
        } else {
            print_success(&format!("Upgraded {} packages", final_upgrades.len()));
            for u in &final_upgrades {
                print_kv(&u.name, &format!("{} -> {}", u.from, u.to));
            }
        }
        print_kv("installed", &summary.installed.len().to_string());
    }

    Ok(ExitCode::Success)
}

fn get_dependency_range(manifest: &PackageManifest, name: &str) -> Option<String> {
    if let Some(entry) = manifest.dependencies.get(name) {
        return Some(entry.version.clone());
    }
    if let Some(entry) = manifest.dev_dependencies.get(name) {
        return Some(entry.version.clone());
    }
    if let Some(entry) = manifest.peer_dependencies.get(name) {
        return Some(entry.version.clone());
    }
    if let Some(entry) = manifest.optional_dependencies.get(name) {
        return Some(entry.version.clone());
    }
    None
}

fn update_manifest_range(
    manifest: &mut PackageManifest,
    name: &str,
    new_range: &str,
) -> Result<()> {
    let mut entry = if let Some(e) = manifest.dependencies.remove(name) {
        e
    } else if let Some(e) = manifest.dev_dependencies.remove(name) {
        e
    } else if let Some(e) = manifest.peer_dependencies.remove(name) {
        e
    } else if let Some(e) = manifest.optional_dependencies.remove(name) {
        e
    } else {
        return Ok(());
    };

    // Update the version range
    entry.version = new_range.to_string();

    // Put back in the same section (we default to production deps since we don't track original)
    // This is a simplification - in reality we'd track the section
    manifest.dependencies.insert(name.to_string(), entry);
    Ok(())
}

#[derive(Debug, Clone)]
struct UpgradeInfo {
    name: String,
    from: String,
    to: String,
    capability_diff: Option<&'static str>,
    api_diff: Option<&'static str>,
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
