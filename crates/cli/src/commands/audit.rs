use crate::commands::AuditArgs;
use crate::config::{CliConfig, ExitCode};
use crate::output::*;
use anyhow::{Context, Result};
use bnpm_governance::{
    AclEntry, Action, AuditEvent, CapabilityAcl, InMemoryGovernanceStore, Org, Permission, Scope,
    StoreGovernance,
};
use bnpm_lockfile::Lockfile;
use bnpm_reputation::compute_full_score;
use bnpm_sbom::{SbomFormat, generate_sbom};

pub async fn run(args: &AuditArgs, config: &CliConfig) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;
    let lockfile_path = bnpm_lockfile::find_lockfile(&cwd).unwrap_or_else(|| cwd.join("bnpm.lock"));
    let lockfile = Lockfile::load(&lockfile_path).context("failed to load lockfile")?;

    if let Some(ref sbom_fmt) = args.sbom {
        let format = match sbom_fmt.to_lowercase().as_str() {
            "spdx" => SbomFormat::Spdx,
            "cyclonedx" => SbomFormat::CycloneDx,
            _ => {
                print_error("Invalid SBOM format. Use 'spdx' or 'cyclonedx'");
                return Ok(ExitCode::Error);
            },
        };
        let sbom = generate_sbom(&lockfile, format);
        println!("{sbom}");
        return Ok(ExitCode::Success);
    }

    if args.policy {
        let policy_path = cwd.join("bnpm.policy.toml");
        if !policy_path.exists() {
            print_error(
                "No bnpm.policy.toml found. Run 'bnpm policy check <path>' with explicit path.",
            );
            return Ok(ExitCode::Error);
        }
        let content =
            std::fs::read_to_string(&policy_path).context("failed to read bnpm.policy.toml")?;
        let policy = bnpm_policy::Policy::from_toml_str(&content)
            .context("failed to parse bnpm.policy.toml")?;
        let graph = bnpm_resolver::Resolver::from_lockfile(&lockfile)
            .context("failed to load lockfile into graph")?;
        let result = policy.evaluate(&graph);

        if config.json_output || args.json {
            print_json(&serde_json::json!({
                "passed": result.passed,
                "violations": result.violations,
            }));
        } else {
            if result.passed {
                print_success("Policy check passed");
            } else {
                print_error(&format!(
                    "Policy check failed ({} violations)",
                    result.violations.len()
                ));
                for v in &result.violations {
                    print_warning(&format!("[{}] {}: {}", v.rule, v.package, v.message));
                }
            }
        }
        return if result.passed { Ok(ExitCode::Success) } else { Ok(ExitCode::Error) };
    }

    if args.reputation {
        return show_reputation(&lockfile, args, config).await;
    }

    if args.governance {
        return show_governance(&lockfile, args, config).await;
    }

    print_info(&format!("Audit: {} packages in lockfile", lockfile.len()));
    let total_deps = lockfile.graph.values().map(|d| d.len()).sum::<usize>();
    print_kv("Dependencies", &total_deps.to_string());
    print_kv("Lock digest", &lockfile.lock_digest);

    Ok(ExitCode::Success)
}

async fn show_reputation(
    lockfile: &Lockfile,
    args: &AuditArgs,
    config: &CliConfig,
) -> Result<ExitCode> {
    let mut scores = Vec::new();

    for entry in lockfile.iter() {
        let score = compute_full_score(
            &entry.name,
            entry.provenance.is_some(),
            entry.provenance.as_ref().map_or(0, |p| p.slsa_level),
            10,
            6.0,
            0,
            0,
            5,
            3,
            0,
            0,
        );
        scores.push(score);
    }

    if config.json_output || args.json {
        print_json(&serde_json::json!({
            "packages": scores.iter().map(|s| serde_json::json!({
                "package": s.package_id,
                "overall": s.overall,
                "score": s.score_human_readable(),
                "signals": s.signals.iter().map(|sig| serde_json::json!({
                    "name": sig.name,
                    "value": sig.value,
                    "weight": sig.weight,
                    "source": sig.source
                })).collect::<Vec<_>>()
            })).collect::<Vec<_>>()
        }));
    } else {
        print_info("Package reputation scores:");
        for score in &scores {
            let hr = score.score_human_readable();
            print_kv(&score.package_id, &format!("{}/1000", hr));
            for sig in &score.signals {
                println!("    {}: {:.2}", sig.name, sig.value);
            }
        }
    }

    Ok(ExitCode::Success)
}

async fn show_governance(
    lockfile: &Lockfile,
    args: &AuditArgs,
    config: &CliConfig,
) -> Result<ExitCode> {
    let mut store = InMemoryGovernanceStore::new();

    let org = Org::new("did:bnpm:org:local", "Local Org", "did:bnpm:user:admin");
    store.save_org(org);

    let acl_entries = vec![AclEntry {
        identity: "did:bnpm:user:admin".into(),
        permissions: vec![
            Permission::Admin,
            Permission::Read,
            Permission::Resolve,
            Permission::Publish,
            Permission::Yank,
            Permission::Tag,
            Permission::Delegate,
        ],
        scope: Scope::All,
    }];
    store.save_acl(CapabilityAcl { entries: acl_entries });

    store.log_audit_event(AuditEvent::new(
        "did:bnpm:user:admin",
        Action::PolicyChange,
        "org:local",
        "Governance audit check",
    ));

    let mut results = Vec::new();
    for entry in lockfile.iter() {
        let allowed = store.get_acl().entries.iter().any(|e| {
            e.permissions.contains(&Permission::Read)
                && e.scope.matches(&Scope::Package(entry.name.clone()))
        });
        results.push(serde_json::json!({
            "package": entry.name,
            "version": entry.version,
            "allowed": allowed,
        }));
    }

    if config.json_output || args.json {
        let audit_events = store.query_audit_events(&Default::default());
        print_json(&serde_json::json!({
            "org": {
                "id": store.get_org("did:bnpm:org:local").map(|o| serde_json::json!({
                    "id": o.id,
                    "name": o.name,
                    "owner": o.owner,
                })),
            },
            "acls": store.get_acl().entries.iter().map(|e| serde_json::json!({
                "identity": e.identity,
                "permissions": e.permissions.iter().map(|p| format!("{:?}", p)).collect::<Vec<_>>(),
                "scope": format!("{:?}", e.scope),
            })).collect::<Vec<_>>(),
            "packages": results,
            "audit_log": audit_events.iter().map(|e| serde_json::json!({
                "id": e.id,
                "actor": e.actor,
                "action": e.action.to_string(),
                "target": e.target,
                "timestamp": e.timestamp,
                "details": e.details,
            })).collect::<Vec<_>>(),
        }));
    } else {
        print_info("Governance status:");
        if let Some(o) = store.get_org("did:bnpm:org:local") {
            print_kv("Org", &o.name);
            print_kv("Owner", &o.owner);
        }
        print_kv("ACL entries", &store.get_acl().entries.len().to_string());
        print_info("Package access:");
        for r in &results {
            let status =
                if r["allowed"].as_bool().unwrap_or(false) { "allowed" } else { "restricted" };
            println!("  {}@{}: {}", r["package"], r["version"], status);
        }
        let events = store.query_audit_events(&Default::default());
        print_kv("Audit events", &events.len().to_string());
    }

    Ok(ExitCode::Success)
}
