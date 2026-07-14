use dash_resolver::ResolvedGraph;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Policy {
    #[serde(rename = "min-slsa-level", default = "default_min_slsa")]
    pub min_slsa_level: u8,
    #[serde(rename = "allow-licenses", default)]
    pub allow_licenses: Vec<String>,
    #[serde(rename = "require-provenance", default = "default_true")]
    pub require_provenance: bool,
    #[serde(rename = "max-cve-severity", default = "default_max_cve")]
    pub max_cve_severity: String,
    #[serde(rename = "version-cooldown", default)]
    pub version_cooldown: Option<String>,
    #[serde(rename = "review-capability-diffs", default = "default_true")]
    pub review_capability_diffs: bool,
    #[serde(rename = "banned-packages", default)]
    pub banned_packages: Vec<String>,
    #[serde(rename = "banned-publishers", default)]
    pub banned_publishers: Vec<String>,
    #[serde(rename = "required-provenance-identities", default)]
    pub required_provenance_identities: Vec<String>,
    #[serde(default)]
    pub rules: Vec<PolicyRule>,
}

fn default_min_slsa() -> u8 {
    1
}

fn default_true() -> bool {
    true
}

fn default_max_cve() -> String {
    "critical".to_string()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyRule {
    pub rule: String,
    pub action: PolicyAction,
    pub severity: PolicySeverity,
    pub condition: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyAction {
    Allow,
    Deny,
    Review,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyResult {
    pub passed: bool,
    pub violations: Vec<PolicyViolation>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyViolation {
    pub rule: String,
    pub package: String,
    pub severity: PolicySeverity,
    pub message: String,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            min_slsa_level: 1,
            allow_licenses: Vec::new(),
            require_provenance: true,
            max_cve_severity: "critical".to_string(),
            version_cooldown: None,
            review_capability_diffs: true,
            banned_packages: Vec::new(),
            banned_publishers: Vec::new(),
            required_provenance_identities: Vec::new(),
            rules: Vec::new(),
        }
    }
}

impl Policy {
    pub fn from_toml_str(input: &str) -> Result<Self, PolicyError> {
        let policy: Policy =
            toml::from_str(input).map_err(|e| PolicyError::Parse(e.to_string()))?;
        Ok(policy)
    }

    pub fn from_json_value(value: serde_json::Value) -> Result<Self, PolicyError> {
        let policy: Policy =
            serde_json::from_value(value).map_err(|e| PolicyError::Parse(e.to_string()))?;
        Ok(policy)
    }

    pub fn evaluate(&self, graph: &ResolvedGraph) -> PolicyResult {
        let mut violations = Vec::new();

        for node in graph.nodes.values() {
            let pkg_name = node.package_id.name.to_string();

            // Check banned packages
            for banned in &self.banned_packages {
                if pkg_name == *banned
                    || pkg_name.contains(banned.trim_end_matches('*').trim_start_matches('*'))
                {
                    violations.push(PolicyViolation {
                        rule: "banned-packages".to_string(),
                        package: pkg_name.clone(),
                        severity: PolicySeverity::Critical,
                        message: format!("package '{}' is banned", pkg_name),
                    });
                }
            }

            // Check provenance requirement
            if self.require_provenance && node.provenance.is_none() {
                violations.push(PolicyViolation {
                    rule: "require-provenance".to_string(),
                    package: pkg_name.clone(),
                    severity: PolicySeverity::High,
                    message: format!("package '{}' is missing provenance", pkg_name),
                });
            }

            // Check SLSA level
            if let Some(ref prov) = node.provenance {
                if prov.slsa_level < self.min_slsa_level {
                    violations.push(PolicyViolation {
                        rule: "min-slsa-level".to_string(),
                        package: pkg_name.clone(),
                        severity: PolicySeverity::Medium,
                        message: format!(
                            "package '{}' has SLSA level {} (required {})",
                            pkg_name, prov.slsa_level, self.min_slsa_level
                        ),
                    });
                }
            }

            // Check banned publishers
            if let Some(ref prov) = node.provenance {
                if self.banned_publishers.iter().any(|bp| prov.publisher.contains(bp)) {
                    violations.push(PolicyViolation {
                        rule: "banned-publishers".to_string(),
                        package: pkg_name.clone(),
                        severity: PolicySeverity::Critical,
                        message: format!(
                            "package '{}' published by banned publisher '{}'",
                            pkg_name, prov.publisher
                        ),
                    });
                }
            }

            // Check license allowlist
            if !self.allow_licenses.is_empty() {
                let pkg_license = node.manifest.license.as_deref().unwrap_or("NOASSERTION");
                if !self.allow_licenses.contains(&pkg_license.to_string()) {
                    violations.push(PolicyViolation {
                        rule: "allow-licenses".to_string(),
                        package: pkg_name.clone(),
                        severity: PolicySeverity::Medium,
                        message: format!(
                            "package '{}' has license '{}' which is not in the allowlist",
                            pkg_name, pkg_license
                        ),
                    });
                }
            }

            // Check required provenance identities
            if !self.required_provenance_identities.is_empty() {
                match node.provenance.as_ref() {
                    Some(prov) => {
                        let has_required = self
                            .required_provenance_identities
                            .iter()
                            .any(|id| prov.publisher.contains(id));
                        if !has_required {
                            violations.push(PolicyViolation {
                                rule: "required-provenance-identities".to_string(),
                                package: pkg_name.clone(),
                                severity: PolicySeverity::High,
                                message: format!(
                                    "package '{}' provenance does not include any required identity",
                                    pkg_name
                                ),
                            });
                        }
                    },
                    None => {
                        violations.push(PolicyViolation {
                            rule: "required-provenance-identities".to_string(),
                            package: pkg_name.clone(),
                            severity: PolicySeverity::High,
                            message: format!(
                                "package '{}' has no provenance to check required identities",
                                pkg_name
                            ),
                        });
                    },
                }
            }
        }

        // Evaluate custom rules
        for rule in &self.rules {
            match rule.action {
                PolicyAction::Deny => {
                    let target =
                        rule.condition.get("package").and_then(|v| v.as_str()).unwrap_or("");
                    let reason = rule
                        .condition
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("blocked by policy rule");
                    violations.push(PolicyViolation {
                        rule: rule.rule.clone(),
                        package: target.to_string(),
                        severity: rule.severity.clone(),
                        message: reason.to_string(),
                    });
                },
                PolicyAction::Review => {
                    // Review rules don't fail but they add warnings
                    let target =
                        rule.condition.get("package").and_then(|v| v.as_str()).unwrap_or("");
                    let reason = rule
                        .condition
                        .get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("needs review");
                    violations.push(PolicyViolation {
                        rule: rule.rule.clone(),
                        package: target.to_string(),
                        severity: rule.severity.clone(),
                        message: reason.to_string(),
                    });
                },
                PolicyAction::Allow => {},
            }
        }

        let passed = violations.is_empty();
        PolicyResult { passed, violations }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use dash_lockfile::Provenance;
    use dash_manifest::PackageManifest;
    use dash_package::{PackageId, PackageName};
    use dash_resolver::ResolvedNode;
    use semver::Version;

    fn make_graph(license: Option<&str>, provenance: Option<Provenance>) -> ResolvedGraph {
        let root = PackageId::new(PackageName::new_unchecked("@acme/app"), Version::new(1, 0, 0));
        let mut graph = ResolvedGraph::new(root.clone());
        let dep_id = PackageId::new(PackageName::new_unchecked("dep"), Version::new(1, 0, 0));
        let manifest = PackageManifest {
            name: "dep".to_string(),
            version: "1.0.0".to_string(),
            license: license.map(|s| s.to_string()),
            ..Default::default()
        };
        let cid: dash_cid::Cid =
            "b3:d4qk6e2jxh27tingubae32rw3teutg6lexe23qisw7gjve6k4qpteyq".parse().unwrap();
        graph.nodes.insert(
            dep_id.clone(),
            ResolvedNode {
                package_id: dep_id.clone(),
                version: Version::new(1, 0, 0),
                manifest,
                cid: Some(cid),
                dependencies: vec![],
                registry: Some("dash:default".into()),
                provenance,
                capabilities: None,
            },
        );
        let root_manifest = PackageManifest {
            name: "@acme/app".to_string(),
            version: "1.0.0".to_string(),
            license: license.map(|s| s.to_string()),
            ..Default::default()
        };
        graph.nodes.insert(
            root.clone(),
            ResolvedNode {
                package_id: root,
                version: Version::new(1, 0, 0),
                manifest: root_manifest,
                cid: None,
                dependencies: vec![],
                registry: None,
                provenance: None,
                capabilities: None,
            },
        );
        graph
    }

    #[test]
    fn test_policy_pass_all_mit() {
        let prov =
            Some(Provenance { log_index: 1, slsa_level: 3, publisher: "did:dash:trusted".into() });
        let graph = make_graph(Some("MIT"), prov);
        let policy = Policy {
            allow_licenses: vec!["MIT".into()],
            min_slsa_level: 2,
            require_provenance: false,
            ..Default::default()
        };
        let result = policy.evaluate(&graph);
        assert!(result.passed);
    }

    #[test]
    fn test_policy_copyleft_violation() {
        let prov =
            Some(Provenance { log_index: 1, slsa_level: 3, publisher: "did:dash:trusted".into() });
        let graph = make_graph(Some("GPL-3.0-only"), prov);
        let policy = Policy {
            allow_licenses: vec!["MIT".into(), "Apache-2.0".into()],
            min_slsa_level: 1,
            ..Default::default()
        };
        let result = policy.evaluate(&graph);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "allow-licenses"));
    }

    #[test]
    fn test_policy_banned_package() {
        let graph = make_graph(Some("MIT"), None);
        let policy = Policy { banned_packages: vec!["dep".into()], ..Default::default() };
        let result = policy.evaluate(&graph);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "banned-packages"));
    }

    #[test]
    fn test_policy_missing_provenance() {
        let graph = make_graph(Some("MIT"), None);
        let policy = Policy {
            require_provenance: true,
            allow_licenses: vec!["MIT".into()],
            ..Default::default()
        };
        let result = policy.evaluate(&graph);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "require-provenance"));
    }

    #[test]
    fn test_policy_slsa_level_too_low() {
        let prov =
            Some(Provenance { log_index: 1, slsa_level: 1, publisher: "did:dash:trusted".into() });
        let graph = make_graph(Some("MIT"), prov);
        let policy =
            Policy { allow_licenses: vec!["MIT".into()], min_slsa_level: 3, ..Default::default() };
        let result = policy.evaluate(&graph);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "min-slsa-level"));
    }

    #[test]
    fn test_policy_banned_publisher() {
        let prov =
            Some(Provenance { log_index: 1, slsa_level: 3, publisher: "did:dash:evil".into() });
        let graph = make_graph(Some("MIT"), prov);
        let policy = Policy {
            banned_publishers: vec!["evil".into()],
            allow_licenses: vec!["MIT".into()],
            ..Default::default()
        };
        let result = policy.evaluate(&graph);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "banned-publishers"));
    }

    #[test]
    fn test_policy_from_toml_str() {
        let toml = r#"
allow-licenses = ["MIT", "Apache-2.0"]
min-slsa-level = 2
require-provenance = true
max-cve-severity = "high"
"#;
        let policy = Policy::from_toml_str(toml).unwrap();
        assert_eq!(policy.allow_licenses, vec!["MIT", "Apache-2.0"]);
        assert_eq!(policy.min_slsa_level, 2);
        assert!(policy.require_provenance);
    }

    #[test]
    fn test_policy_from_json() {
        let json = serde_json::json!({
            "allow-licenses": ["MIT"],
            "min-slsa-level": 3,
            "banned-packages": ["bad-pkg"]
        });
        let policy = Policy::from_json_value(json).unwrap();
        assert_eq!(policy.min_slsa_level, 3);
        assert_eq!(policy.banned_packages, vec!["bad-pkg"]);
    }

    #[test]
    fn test_policy_no_violations() {
        let prov =
            Some(Provenance { log_index: 1, slsa_level: 3, publisher: "did:dash:trusted".into() });
        let graph = make_graph(Some("MIT"), prov);
        let policy = Policy { require_provenance: false, ..Default::default() };
        let result = policy.evaluate(&graph);
        assert!(result.passed);
    }

    #[test]
    fn test_custom_rule_deny() {
        let graph = make_graph(Some("MIT"), None);
        let policy = Policy {
            rules: vec![PolicyRule {
                rule: "custom-deny".to_string(),
                action: PolicyAction::Deny,
                severity: PolicySeverity::High,
                condition: serde_json::json!({"package": "dep", "reason": "custom block"}),
            }],
            ..Default::default()
        };
        let result = policy.evaluate(&graph);
        assert!(!result.passed);
        assert!(result.violations.iter().any(|v| v.rule == "custom-deny"));
    }
}
