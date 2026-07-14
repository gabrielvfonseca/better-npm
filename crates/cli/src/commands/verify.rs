use crate::commands::VerifyArgs;
use crate::config::{CliConfig, ExitCode};
use crate::output::*;
use anyhow::{Context, Result};
use dash_cid::Cid;
use dash_lockfile::{Lockfile, find_lockfile};
use dash_store::CasStore;
use std::sync::Arc;

pub async fn run(args: &VerifyArgs, config: &CliConfig) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;

    let lockfile_path = find_lockfile(&cwd).unwrap_or_else(|| cwd.join("dash.lock"));
    let lockfile = Lockfile::load(&lockfile_path).context("failed to load lockfile")?;

    let store = Arc::new(CasStore::new(config.store_dir()).await.context("failed to open store")?);

    let mut verified = 0;
    let mut failed = 0;
    let mut failed_packages = Vec::new();
    let mut provenance_results = Vec::new();

    for entry in lockfile.iter() {
        let cid: Cid = entry.cid.parse().context("invalid CID in lockfile")?;

        let exists = store.has_capsule(&cid).await.context("failed to check store")?;

        let integrity_ok = if exists {
            store.verify(&cid).await.context("failed to verify capsule")?
        } else {
            false
        };

        if integrity_ok {
            verified += 1;
        } else {
            failed += 1;
            failed_packages.push(serde_json::json!({
                "name": entry.name,
                "version": entry.version,
                "cid": entry.cid,
                "reason": if exists { "integrity check failed" } else { "missing from store" }
            }));
        }

        if let Some(ref provenance) = entry.provenance {
            provenance_results.push(serde_json::json!({
                "name": entry.name,
                "version": entry.version,
                "logIndex": provenance.log_index,
                "slsaLevel": provenance.slsa_level,
                "publisher": provenance.publisher,
            }));
        }
    }

    let mut checkpoint_info = serde_json::Value::Null;
    let witnesses: Vec<String> = Vec::new();

    if args.checkpoint {
        let checkpoint = serde_json::json!({
            "rootHash": "pending",
            "treeSize": 0,
            "timestamp": chrono::Utc::now().timestamp(),
        });
        if config.json_output {
            checkpoint_info = checkpoint;
        } else {
            print_info("Checkpoint: transparency log verification requires a running registry");
            print_kv("Root hash", "pending (start dash-gateway for checkpoint)");
        }
    }

    if args.strict {
        if config.json_output {
            // witnesses remain empty for MVP
        } else {
            print_warning(
                "--strict: witness verification requires a running registry with witnesses",
            );
        }
    }

    if config.json_output {
        print_json(&serde_json::json!({
            "verified": verified,
            "failed": failed,
            "checkpoint": checkpoint_info,
            "witnesses": witnesses,
            "failedPackages": failed_packages,
            "provenance": provenance_results,
        }));
    } else {
        if failed == 0 {
            print_success(&format!("Verified {} packages", verified));
        } else {
            print_error(&format!("Verification failed: {} packages failed", failed));
            for pkg in &failed_packages {
                print_kv("  -", &format!("{}@{} ({})", pkg["name"], pkg["version"], pkg["reason"]));
            }
        }
    }

    if failed > 0 { Ok(ExitCode::Error) } else { Ok(ExitCode::Success) }
}
