//! `bnpm add` command

use crate::commands::AddArgs;
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
    DependencyEntry, Manifest, PackageManifest, find_manifest,
    schema::Manifest as SchemaManifest,
};
use bnpm_resolver::RegistryConfig;
use bnpm_resolver::ResolveOptions;
use bnpm_resolver::Resolver;
use bnpm_store::CasStore;
use semver::VersionReq;
use std::sync::Arc;
use tokio::fs;

pub async fn run(args: &AddArgs, config: &CliConfig) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;

    // Find manifest
    let manifest_path = find_manifest(&cwd)
        .context("no manifest found (bnpm.toml, bnpm.json, or package.json)")?;

    // Load manifest
    let mut manifest: PackageManifest = {
        let m = Manifest::load(&manifest_path).context("failed to load manifest")?;
        m.parsed
    };

    // Parse spec: name@range or just name
    let (name, range) = parse_spec(&args.spec);
    let range = range.unwrap_or_else(|| "^".to_string());

    // Validate range if --exact
    let version_req = if args.exact {
        // For exact, we need to resolve to a specific version first
        range.clone()
    } else {
        VersionReq::parse(&range)
            .context(format!("invalid version range: {}", range))?
            .to_string()
    };

    // Add dependency to correct section
    let mut dep_entry = DependencyEntry::new(&version_req);
    if args.dev {
        dep_entry = dep_entry.dev();
    }

    if args.peer {
        manifest.peer_dependencies.insert(name.clone(), dep_entry.clone());
    } else if args.optional {
        manifest.optional_dependencies.insert(name.clone(), dep_entry.clone());
    } else if args.dev {
        manifest.dev_dependencies.insert(name.clone(), dep_entry);
    } else {
        manifest.dependencies.insert(name.clone(), dep_entry);
    }

    // Write updated manifest
    if manifest_path.extension().and_then(|s| s.to_str()) == Some("toml") {
        let json_str = serde_json::to_string_pretty(&manifest)?;
        let bnpm_manifest = SchemaManifest::from_json_str(&json_str)?;
        let toml_str = bnpm_manifest.to_toml_string()?;
        fs::write(&manifest_path, toml_str).await?;
    } else {
        let json_str = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, json_str).await?;
    }

    // Find lockfile
    let lockfile_path = bnpm_lockfile::find_lockfile(&cwd).unwrap_or_else(|| cwd.join("bnpm.lock"));

    // Load old lockfile
    let _old_lockfile = if lockfile_path.exists() {
        Lockfile::load(&lockfile_path).context("failed to load lockfile")?
    } else {
        Lockfile::with_root(format!("{}@{}", manifest.name, manifest.version))
    };

    // Re-resolve with new manifest
    let resolver = create_resolver(&manifest, config)?;
    let resolve_options = ResolveOptions::new(manifest.clone())
        .with_cache(create_cache(&CacheConfig::default()))
        .offline(config.offline)
        .max_parallel(10);
    let resolved_graph = resolver
        .resolve(resolve_options)
        .await
        .context("failed to resolve dependencies")?;

    // Create new lockfile
    let new_lockfile =
        Resolver::to_lockfile(&resolved_graph).context("failed to create lockfile")?;

    // Handle --why
    if args.why || config.why {
        let trace = resolved_graph.why(&name);
        if config.json_output {
            print_json(&serde_json::json!({
                "package": name,
                "resolutionTrace": trace.iter().map(|e| serde_json::json!({
                    "requiredBy": e.required_by.to_string(),
                    "range": e.range,
                    "selected": e.selected.to_string(),
                    "kind": format!("{:?}", e.kind).to_lowercase()
                })).collect::<Vec<_>>()
            }));
        } else {
            print_info(&format!("Resolution trace for {}:", name));
            for edge in trace {
                print_kv(
                    "  via",
                    &format!(
                        "{} requires {}@{} -> {}@{}",
                        edge.required_by, name, edge.range, name, edge.selected
                    ),
                );
            }
        }
    }

    // Skip install if --no-install
    if !args.no_install {
        // Write new lockfile
        new_lockfile.write_to_file(&lockfile_path)?;

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
            store.clone() as Arc<dyn bnpm_store::StoreBackend>,
            LinkerConfig { bin_dir: installer_config.bin_dir.clone(), ..Default::default() },
        );
        let resolver2 = create_resolver(&manifest, config)?;
        let installer =
            Installer::new(resolver2, downloader, linker, store, installer_config, cache);

        let summary = installer.install_from_lockfile(&new_lockfile).await?;

        if !config.json_output && !config.quiet {
            print_success(&format!("Added {} packages", summary.installed.len()));
        }
    } else {
        // Just write lockfile
        new_lockfile.write_to_file(&lockfile_path)?;
    }

    // Output
    if config.json_output {
        let added_info = new_lockfile.get(&name).map(|p| {
            serde_json::json!({
                "name": p.name,
                "version": p.version,
                "cid": p.cid
            })
        });
        let added_vec = added_info.into_iter().collect::<Vec<_>>();
        let trace = resolved_graph.why(&name);
        print_json(&serde_json::json!({
            "added": added_vec,
            "lockfile": lockfile_path.display().to_string(),
            "resolutionTrace": trace.iter().map(|e| serde_json::json!({
                "package": name,
                "requiredBy": [e.required_by.to_string()],
                "constraint": e.range,
                "selected": e.selected.to_string()
            })).collect::<Vec<_>>()
        }));
    } else if !config.quiet && !args.no_install {
        if let Some(entry) = new_lockfile.get(&name) {
            print_success(&format!("Added {}@{}", name, entry.version));
        }
    }

    Ok(ExitCode::Success)
}

fn parse_spec(spec: &str) -> (String, Option<String>) {
    // Handle scoped packages that start with @
    if let Some(stripped) = spec.strip_prefix('@') {
        // Find the second @ which separates name from version
        if let Some(second_at) = stripped.find('@') {
            // @scope/name@version
            let name = spec[..1 + second_at].to_string();
            let range = spec[1 + second_at + 1..].to_string();
            (name, Some(range))
        } else {
            // Just @scope/name without version
            (spec.to_string(), None)
        }
    } else if let Some(at_pos) = spec.find('@') {
        // Regular name@range
        let name = spec[..at_pos].to_string();
        let range = spec[at_pos + 1..].to_string();
        (name, Some(range))
    } else {
        (spec.to_string(), None)
    }
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

    #[test]
    fn test_parse_spec() {
        assert_eq!(parse_spec("zod"), ("zod".to_string(), None));
        assert_eq!(parse_spec("zod@^3.23"), ("zod".to_string(), Some("^3.23".to_string())));
        assert_eq!(parse_spec("@scope/pkg"), ("@scope/pkg".to_string(), None));
        assert_eq!(
            parse_spec("@scope/pkg@^1.0"),
            ("@scope/pkg".to_string(), Some("^1.0".to_string()))
        );
    }
}
