//! `bnpm agent` command — stub for MVP.

use crate::commands::{AgentArgs, AgentSubcommand};
use crate::config::{CliConfig, ExitCode};
use crate::output::*;
use anyhow::Result;

pub async fn run(args: &AgentArgs, config: &CliConfig) -> Result<ExitCode> {
    if config.json_output {
        let output = serde_json::json!({
            "message": "agent mode requires @better-npm/agent SDK",
            "subcommand": format!("{:?}", args.subcommand),
            "docs": "https://bnpm.dev/docs/agent"
        });
        print_json(&output);
    } else {
        match &args.subcommand {
            AgentSubcommand::Select { query } => {
                print_info(&format!("Agent select: scanning for packages matching '{}'...", query));
            },
            AgentSubcommand::Explain { package } => {
                print_info(&format!("Agent explain: fetching provenance for '{}'...", package));
            },
            AgentSubcommand::PlanUpgrade { goal } => {
                let goal_str = goal.as_deref().unwrap_or("minimal-safe");
                print_info(&format!(
                    "Agent plan-upgrade: computing upgrade plan for goal '{}'...",
                    goal_str
                ));
            },
            AgentSubcommand::Audit => {
                print_info("Agent audit: running AI-native audit...");
            },
            AgentSubcommand::Compose { tool_spec } => {
                if let Some(spec) = tool_spec {
                    print_info(&format!(
                        "Agent compose: finding packages matching tools '{}'...",
                        spec
                    ));
                } else {
                    print_info("Agent compose: finding packages matching tool needs...");
                }
            },
        }
        println!();
        print_warning("agent mode requires @better-npm/agent SDK — npm install @better-npm/agent");
        println!();
        print_info("The @better-npm/agent SDK provides programmatic access to:");
        print_kv("select", "search packages by intent + constraints");
        print_kv("explain", "explain why a package is installed");
        print_kv("plan-upgrade", "compute minimal safe upgrade plan");
        print_kv("audit", "return machine-readable audit");
        print_kv("compose", "propose packages matching tool needs");
    }

    Ok(ExitCode::Success)
}
