use crate::commands::PolicyCheckArgs;
use crate::config::{CliConfig, ExitCode};
use crate::output::*;
use anyhow::{Context, Result};
use dash_lockfile::Lockfile;
use dash_policy::Policy;

pub async fn run(args: &PolicyCheckArgs, config: &CliConfig) -> Result<ExitCode> {
    let cwd = std::env::current_dir().context("failed to get current directory")?;

    let policy_path = if args.policy_path.as_os_str().is_empty()
        || args.policy_path.to_string_lossy() == "dash.policy.toml"
    {
        cwd.join("dash.policy.toml")
    } else {
        args.policy_path.clone()
    };

    if !policy_path.exists() {
        print_error(&format!("Policy file not found: {}", policy_path.display()));
        return Ok(ExitCode::Error);
    }

    let content = std::fs::read_to_string(&policy_path).context("failed to read policy file")?;
    let policy = Policy::from_toml_str(&content).context("failed to parse policy file")?;

    let lockfile_path = dash_lockfile::find_lockfile(&cwd).unwrap_or_else(|| cwd.join("dash.lock"));
    let lockfile = Lockfile::load(&lockfile_path).context("failed to load lockfile")?;

    let graph = dash_resolver::Resolver::from_lockfile(&lockfile)
        .context("failed to load lockfile into graph")?;
    let result = policy.evaluate(&graph);

    if config.json_output {
        print_json(&serde_json::json!({
            "passed": result.passed,
            "violations": result.violations,
            "policy": policy,
        }));
    } else {
        if result.passed {
            print_success("Policy check passed");
        } else {
            print_error(&format!("Policy check FAILED ({} violation(s))", result.violations.len()));
            for v in &result.violations {
                print_warning(&format!(
                    "  [{:?}] {}: {} ({})",
                    v.severity, v.rule, v.message, v.package
                ));
            }
        }
    }

    Ok(if result.passed { ExitCode::Success } else { ExitCode::Error })
}
