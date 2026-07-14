use dash_governance::{CapabilityAcl, Permission, Scope};
use dash_policy::Policy;
use dash_proto::dpr_v1::{Binding, Provenance};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Wrapper around Scope to allow extension methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeWrapper(pub Scope);

impl ScopeWrapper {
    pub fn to_string(&self) -> String {
        match &self.0 {
            Scope::All => "*".to_string(),
            Scope::Namespace(ns) => ns.clone(),
            Scope::Package(pkg) => pkg.clone(),
            Scope::Version(ver) => ver.clone(),
        }
    }
}

impl From<Scope> for ScopeWrapper {
    fn from(scope: Scope) -> Self {
        ScopeWrapper(scope)
    }
}

impl From<ScopeWrapper> for Scope {
    fn from(wrapper: ScopeWrapper) -> Self {
        wrapper.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyBoundaryConfig {
    pub enable_policy_enforcement: bool,
    pub enable_acl_enforcement: bool,
    pub reject_on_policy_violation: bool,
    pub reject_on_acl_violation: bool,
    pub log_violations: bool,
}

impl Default for PolicyBoundaryConfig {
    fn default() -> Self {
        Self {
            enable_policy_enforcement: true,
            enable_acl_enforcement: true,
            reject_on_policy_violation: true,
            reject_on_acl_violation: true,
            log_violations: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub violation_type: PolicyViolationType,
    pub package: String,
    pub version: String,
    pub message: String,
    pub severity: ViolationSeverity,
}

impl fmt::Display for PolicyViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} v{} - {} ({:?})", self.violation_type, self.package, self.version, self.message, self.severity)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyViolationType {
    PolicyDeny,
    PolicyReviewRequired,
    AclDenied,
    ProvenanceMissing,
    SlsaLevelInsufficient,
    LicenseNotAllowed,
    BannedPackage,
    BannedPublisher,
    CapabilityDiffUnreviewed,
    Custom(String),
}

impl fmt::Display for PolicyViolationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyViolationType::PolicyDeny => write!(f, "policy deny"),
            PolicyViolationType::PolicyReviewRequired => write!(f, "policy review required"),
            PolicyViolationType::AclDenied => write!(f, "ACL denied"),
            PolicyViolationType::ProvenanceMissing => write!(f, "provenance missing"),
            PolicyViolationType::SlsaLevelInsufficient => write!(f, "SLSA level insufficient"),
            PolicyViolationType::LicenseNotAllowed => write!(f, "license not allowed"),
            PolicyViolationType::BannedPackage => write!(f, "banned package"),
            PolicyViolationType::BannedPublisher => write!(f, "banned publisher"),
            PolicyViolationType::CapabilityDiffUnreviewed => write!(f, "capability diff unreviewed"),
            PolicyViolationType::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationContext {
    pub name: String,
    pub version: String,
    pub cid: String,
    pub log_index: u64,
    pub provenance: Option<Provenance>,
}

pub struct PolicyBoundary {
    config: PolicyBoundaryConfig,
    policy: Option<Policy>,
    acl: Option<CapabilityAcl>,
}

impl PolicyBoundary {
    pub fn new(config: PolicyBoundaryConfig, policy: Option<Policy>, acl: Option<CapabilityAcl>) -> Self {
        Self { config, policy, acl }
    }

    pub fn evaluate(&self, binding: &Binding, identity: &str, scope: &ScopeWrapper) -> Result<(), PolicyViolation> {
        if self.config.enable_policy_enforcement {
            if let Some(ref policy) = self.policy {
                self.evaluate_policy(binding, policy)?;
            }
        }

        if self.config.enable_acl_enforcement {
            if let Some(ref acl) = self.acl {
                self.evaluate_acl(identity, scope, acl)?;
            }
        }

        Ok(())
    }

    pub fn evaluate_with_wrapper(&self, binding: &Binding, identity: &str, scope: &ScopeWrapper) -> Result<(), PolicyViolation> {
        if self.config.enable_policy_enforcement {
            if let Some(ref policy) = self.policy {
                self.evaluate_policy(binding, policy)?;
            }
        }

        if self.config.enable_acl_enforcement {
            if let Some(ref acl) = self.acl {
                self.evaluate_acl(identity, scope, acl)?;
            }
        }

        Ok(())
    }

    fn evaluate_policy(&self, binding: &Binding, policy: &Policy) -> Result<(), PolicyViolation> {
        let package = &binding.version;

        if policy.banned_packages.iter().any(|b| {
            package == b || package.contains(&b.trim_end_matches('*').trim_start_matches('*'))
        }) {
            return Err(PolicyViolation {
                violation_type: PolicyViolationType::BannedPackage,
                package: package.to_string(),
                version: binding.version.clone(),
                message: format!("Package '{}' is banned by policy", package),
                severity: ViolationSeverity::Critical,
            });
        }

        if let Some(ref prov) = binding.provenance {
            if policy.banned_publishers.iter().any(|bp| prov.identity.contains(bp)) {
                return Err(PolicyViolation {
                    violation_type: PolicyViolationType::BannedPublisher,
                    package: package.to_string(),
                    version: binding.version.clone(),
                    message: format!("Package '{}' published by banned publisher", package),
                    severity: ViolationSeverity::Critical,
                });
            }

            if policy.min_slsa_level > 0 {
                let prov_slsa = prov.slsa_level.parse::<u8>().unwrap_or(0);
                if prov_slsa < policy.min_slsa_level {
                    return Err(PolicyViolation {
                        violation_type: PolicyViolationType::SlsaLevelInsufficient,
                        package: package.to_string(),
                        version: binding.version.clone(),
                        message: format!(
                            "Package '{}' has SLSA level {} (required {})",
                            package, prov_slsa, policy.min_slsa_level
                        ),
                        severity: ViolationSeverity::Medium,
                    });
                }
            }

            if !policy.required_provenance_identities.is_empty() {
                let has_required = policy.required_provenance_identities.iter().any(|id| prov.identity.contains(id));
                if !has_required {
                    return Err(PolicyViolation {
                        violation_type: PolicyViolationType::Custom("required-provenance-identity".to_string()),
                        package: package.to_string(),
                        version: binding.version.clone(),
                        message: format!("Package '{}' provenance does not include required identity", package),
                        severity: ViolationSeverity::High,
                    });
                }
            }
        } else if policy.require_provenance {
            return Err(PolicyViolation {
                violation_type: PolicyViolationType::ProvenanceMissing,
                package: package.to_string(),
                version: binding.version.clone(),
                message: format!("Package '{}' is missing provenance", package),
                severity: ViolationSeverity::High,
            });
        }

        for rule in &policy.rules {
            match rule.action {
                dash_policy::PolicyAction::Deny => {
                    return Err(PolicyViolation {
                        violation_type: PolicyViolationType::PolicyDeny,
                        package: package.to_string(),
                        version: binding.version.clone(),
                        message: rule.condition.get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("blocked by policy rule")
                            .to_string(),
                        severity: match rule.severity {
                            dash_policy::PolicySeverity::Critical => ViolationSeverity::Critical,
                            dash_policy::PolicySeverity::High => ViolationSeverity::High,
                            dash_policy::PolicySeverity::Medium => ViolationSeverity::Medium,
                            dash_policy::PolicySeverity::Low => ViolationSeverity::Low,
                        },
                    });
                }
                dash_policy::PolicyAction::Review => {
                    return Err(PolicyViolation {
                        violation_type: PolicyViolationType::PolicyReviewRequired,
                        package: package.to_string(),
                        version: binding.version.clone(),
                        message: rule.condition.get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("needs review")
                            .to_string(),
                        severity: match rule.severity {
                            dash_policy::PolicySeverity::Critical => ViolationSeverity::Critical,
                            dash_policy::PolicySeverity::High => ViolationSeverity::High,
                            dash_policy::PolicySeverity::Medium => ViolationSeverity::Medium,
                            dash_policy::PolicySeverity::Low => ViolationSeverity::Low,
                        },
                    });
                }
                dash_policy::PolicyAction::Allow => {}
            }
        }

        Ok(())
    }

    fn evaluate_acl(&self, identity: &str, scope: &ScopeWrapper, acl: &CapabilityAcl) -> Result<(), PolicyViolation> {
        use dash_governance::check_permission;

        let has_resolve = check_permission(acl, identity, &Permission::Resolve, &scope.0);

        if !has_resolve {
            return Err(PolicyViolation {
                violation_type: PolicyViolationType::AclDenied,
                package: scope.to_string(),
                version: String::new(),
                message: format!("Identity '{}' does not have resolve permission for scope {:?}", identity, scope.0),
                severity: ViolationSeverity::High,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dash_governance::Scope;
    use dash_policy::Policy;

    #[test]
    fn test_policy_boundary_allows_when_no_policy() {
        let boundary = PolicyBoundary::new(PolicyBoundaryConfig::default(), None, None);
        let binding = Binding {
            version: "1.0.0".to_string(),
            cid: "b3:test".to_string(),
            log_index: 1,
            inclusion_proof: None,
            provenance: None,
        };
        let result = boundary.evaluate(&binding, "did:dash:user", &Scope::All.into());
        assert!(result.is_ok());
    }

    #[test]
    fn test_policy_boundary_banned_package() {
        let policy = Policy {
            banned_packages: vec!["1.0.0".to_string()],
            ..Default::default()
        };
        let boundary = PolicyBoundary::new(PolicyBoundaryConfig::default(), Some(policy), None);
        let binding = Binding {
            version: "1.0.0".to_string(),
            cid: "b3:test".to_string(),
            log_index: 1,
            inclusion_proof: None,
            provenance: None,
        };
        let result = boundary.evaluate(&binding, "did:dash:user", &Scope::All.into());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().violation_type, PolicyViolationType::BannedPackage));
    }
}