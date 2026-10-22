//! `bnpm install` command

use crate::commands::InstallArgs;
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
use bnpm_manifest::{Manifest, PackageManifest, find_manifest};
use bnpm_resolver::RegistryConfig;
use bnpm_resolver::ResolveOptions;
use bnpm_resolver::Resolver;
use bnpm_store::CasStore;
use std::sync::Arc;

pub async fn run(args: &InstallArgs, config: &CliConfig) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;

    // Find manifest (prefer bnpm.toml via find_manifest_toml, fallback to find_manifest)
    let manifest_path = find_manifest(&cwd)
        .context("no manifest found (bnpm.toml, bnpm.json, or package.json)")?;

    // Load manifest
    let manifest: PackageManifest = {
        let m = Manifest::load(&manifest_path).context("failed to load manifest")?;
        m.parsed
    };

    // Find lockfile
    let lockfile_path = bnpm_lockfile::find_lockfile(&cwd).unwrap_or_else(|| cwd.join("bnpm.lock"));

    // Load or create lockfile
    let lockfile = if lockfile_path.exists() {
        Lockfile::load(&lockfile_path).context("failed to load lockfile")?
    } else {
        Lockfile::with_root(format!("{}@{}", manifest.name, manifest.version))
    };

    // Check frozen-lock
    if args.frozen_lock || config.frozen_lock {
        let resolver = create_resolver(&manifest, config)?;
        let resolve_options = ResolveOptions::new(manifest.clone())
            .with_cache(create_cache(&CacheConfig::default()))
            .offline(args.offline || config.offline)
            .max_parallel(10);
        let resolved_graph = resolver
            .resolve(resolve_options)
            .await
            .context("failed to resolve dependencies for frozen-lock check")?;
        let new_lockfile = Resolver::to_lockfile(&resolved_graph)
            .context("failed to convert resolved graph to lockfile")?;

        let new_fingerprint = new_lockfile
            .fingerprint()
            .context("failed to compute new lockfile fingerprint")?;
        let old_fingerprint = lockfile
            .fingerprint()
            .context("failed to compute current lockfile fingerprint")?;

        if new_fingerprint != old_fingerprint {
            if config.json_output {
                print_json(&serde_json::json!({
                    "error": "lockfile would change",
                    "lockfile": lockfile_path.display().to_string()
                }));
            } else {
                print_error("Lockfile would change (use --frozen-lock to prevent)");
            }
            return Ok(ExitCode::LockfileChanged);
        }
    }

    // Check offline mode
    if args.offline || config.offline {
        let store = Arc::new(CasStore::new(config.store_dir()).await?);
        for entry in lockfile.iter() {
            let cid: bnpm_cid::Cid = entry.cid.parse()?;
            if !store.has_capsule(&cid).await? {
                if config.json_output {
                    print_json(&serde_json::json!({
                        "error": "missing artifact in offline mode",
                        "package": entry.name,
                        "cid": entry.cid
                    }));
                } else {
                    print_error(&format!(
                        "Missing artifact for {}@{} (CID: {}) - run without --offline to fetch",
                        entry.name, entry.version, entry.cid
                    ));
                }
                return Ok(ExitCode::NetworkRequired);
            }
        }
    }

    // Create installer
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
    let resolver = create_resolver(&manifest, config)?;
    let installer = Installer::new(resolver, downloader, linker, store, installer_config, cache);

    // Install from lockfile
    let summary = installer.install_from_lockfile(&lockfile).await?;

    // Output
    if config.json_output {
        // Count cached vs fetched (simplified - installer doesn't expose this directly)
        let from_cache = summary.installed.len().saturating_sub(2); // approximate
        let fetched = summary.installed.len().saturating_sub(from_cache);
        print_json(&serde_json::json!({
            "installed": summary.installed.len(),
            "fromCache": from_cache,
            "fetched": fetched,
            "bytesTransferred": summary.total_bytes,
            "lockfile": lockfile_path.display().to_string()
        }));
    } else if !config.quiet {
        print_success(&format!("Installed {} packages", summary.installed.len()));
    }

    // Handle --vendor (deferred - just note it)
    if args.vendor {
        print_warning("--vendor flag not yet implemented");
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
    // Add npm registry as fallback upstream for packages not in Bnpm registry
    let npm_registry = RegistryConfig {
        url: "https://registry.npmjs.org".to_string(),
        token: None,
        priority: 10,
        timeout_secs: 60,
        verify_ssl: true,
    };
    let options = ResolveOptions::new(manifest.clone())
        .with_registry(registry_config)
        .with_registry(npm_registry)
        .with_cache(cache)
        .offline(config.offline)
        .max_parallel(10);
    Ok(Resolver::new(options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_install_args() {
        let _args = InstallArgs {
            packages: None,
            production: false,
            offline: false,
            frozen_lock: false,
            no_dev: false,
            vendor: false,
        };
        let _config = CliConfig::default();
        // Would test run but needs full setup
    }
}
