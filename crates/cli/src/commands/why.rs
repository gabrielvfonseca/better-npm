//! `bnpm why` command

use crate::commands::WhyArgs;
use crate::config::{CliConfig, ExitCode};
use crate::output::*;
use anyhow::{Context, Result};
use bnpm_cache::{CacheConfig, create_cache};
use bnpm_lockfile::Lockfile;
use bnpm_manifest::{Manifest, PackageManifest, find_manifest};
use bnpm_resolver::RegistryConfig;
use bnpm_resolver::ResolveOptions;
use bnpm_resolver::Resolver;
use serde_json;

pub async fn run(args: &WhyArgs, config: &CliConfig) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;

    // Find manifest
    let manifest_path = find_manifest(&cwd)
        .context("no manifest found")?;

    // Load manifest
    let manifest: PackageManifest = {
        let m = Manifest::load(&manifest_path)?;
        m.parsed
    };

    // Load lockfile
    let lockfile_path = bnpm_lockfile::find_lockfile(&cwd).unwrap_or_else(|| cwd.join("bnpm.lock"));
    let lockfile = Lockfile::load(&lockfile_path).context("failed to load lockfile")?;

    // Find package in lockfile
    let entry = lockfile
        .get(&args.package)
        .context(format!("package '{}' not found in lockfile", args.package))?;

    // Resolve graph for --graph
    let resolver = create_resolver(&manifest, config)?;
    let resolve_options = ResolveOptions::new(manifest.clone())
        .with_cache(create_cache(&CacheConfig::default()))
        .offline(config.offline)
        .max_parallel(10);
    let resolved_graph = resolver
        .resolve(resolve_options)
        .await
        .context("failed to resolve dependencies")?;

    // Build output
    if config.json_output {
        let mut output = serde_json::json!({
            "package": {
                "name": entry.name,
                "version": entry.version,
                "cid": entry.cid
            }
        });

        if args.provenance {
            if let Some(ref prov) = entry.provenance {
                output["provenance"] = serde_json::json!({
                    "publisher": prov.publisher,
                    "logIndex": prov.log_index,
                    "slsaLevel": prov.slsa_level
                });
            } else {
                output["provenance"] = serde_json::Value::Null;
            }
        }

        if args.graph {
            let trace = resolved_graph.why(&args.package);
            output["graphPath"] = serde_json::json!(
                trace.iter().map(|e| e.required_by.to_string()).collect::<Vec<_>>()
            );
            output["resolutionTrace"] = serde_json::json!(
                trace
                    .iter()
                    .map(|e| serde_json::json!({
                        "requiredBy": e.required_by.to_string(),
                        "range": e.range,
                        "selected": e.selected.to_string(),
                        "kind": format!("{:?}", e.kind).to_lowercase()
                    }))
                    .collect::<Vec<_>>()
            );
        }

        if args.capabilities {
            if let Some(ref caps) = entry.capabilities {
                output["capabilities"] = serde_json::to_value(caps)?;
            } else {
                output["capabilities"] = serde_json::Value::Null;
            }
        }

        print_json(&output);
    } else {
        print_info(&format!("{}@{} ({})", entry.name, entry.version, entry.cid));

        if args.provenance {
            if let Some(ref prov) = entry.provenance {
                print_kv("Publisher", &prov.publisher);
                print_kv("Log Index", &prov.log_index.to_string());
                print_kv("SLSA Level", &prov.slsa_level.to_string());
            } else {
                print_warning("No provenance information available (requires M3 transparency log)");
            }
        }

        if args.graph {
            let trace = resolved_graph.why(&args.package);
            if trace.is_empty() {
                print_warning("No resolution path found");
            } else {
                print_info("Dependency path:");
                for edge in trace {
                    print_kv(
                        "  via",
                        &format!(
                            "{} requires {}@{} -> {}@{}",
                            edge.required_by, args.package, edge.range, args.package, edge.selected
                        ),
                    );
                }
            }
        }

        if args.capabilities {
            if let Some(ref caps) = entry.capabilities {
                print_info("Capabilities:");
                if let Some(ref net) = caps.net {
                    print_kv("  net", &format!("{:?}", net));
                }
                if let Some(ref fs_read) = caps.fs_read {
                    print_kv("  fs-read", &format!("{:?}", fs_read));
                }
                if let Some(ref fs_write) = caps.fs_write {
                    print_kv("  fs-write", &format!("{:?}", fs_write));
                }
                if let Some(ref env) = caps.env {
                    print_kv("  env", &format!("{:?}", env));
                }
                if let Some(subprocess) = caps.subprocess {
                    print_kv("  subprocess", &subprocess.to_string());
                }
            } else {
                print_info("No capabilities declared");
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
