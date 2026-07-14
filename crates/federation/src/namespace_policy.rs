use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NamespacePolicy {
    pub prefix: String,
    pub delegate_to_upstream: bool,
    pub local_only: bool,
    pub allowed_upstreams: Vec<String>,
}

impl NamespacePolicy {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            delegate_to_upstream: true,
            local_only: false,
            allowed_upstreams: Vec::new(),
        }
    }

    pub fn local_only(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            delegate_to_upstream: false,
            local_only: true,
            allowed_upstreams: Vec::new(),
        }
    }

    pub fn delegate_to_upstream(mut self, delegate: bool) -> Self {
        self.delegate_to_upstream = delegate;
        if !delegate {
            self.local_only = true;
        }
        self
    }

    pub fn allowed_upstreams(mut self, upstreams: Vec<String>) -> Self {
        self.allowed_upstreams = upstreams;
        self
    }

    pub fn matches(&self, namespace: &str) -> bool {
        namespace == self.prefix || namespace.starts_with(&format!("{}/", self.prefix))
    }

    pub fn matches_upstream(&self, upstream_name: &str) -> bool {
        self.allowed_upstreams.is_empty() || self.allowed_upstreams.iter().any(|u| u == upstream_name)
    }

    pub fn is_local_only(&self) -> bool {
        self.local_only
    }

    pub fn should_delegate(&self) -> bool {
        self.delegate_to_upstream && !self.local_only
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NamespacePolicyConfig {
    pub policies: Vec<NamespacePolicy>,
    pub default_delegate: bool,
}

impl NamespacePolicyConfig {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
            default_delegate: true,
        }
    }

    pub fn add_policy(&mut self, policy: NamespacePolicy) {
        self.policies.push(policy);
    }

    pub fn find_policy(&self, namespace: &str) -> Option<&NamespacePolicy> {
        self.policies.iter().find(|p| p.matches(namespace))
    }

    pub fn should_delegate(&self, namespace: &str) -> bool {
        if let Some(policy) = self.find_policy(namespace) {
            policy.should_delegate()
        } else {
            self.default_delegate
        }
    }

    pub fn is_local_only(&self, namespace: &str) -> bool {
        if let Some(policy) = self.find_policy(namespace) {
            policy.is_local_only()
        } else {
            false
        }
    }

    pub fn allowed_upstreams(&self, namespace: &str) -> Option<&Vec<String>> {
        self.find_policy(namespace).map(|p| &p.allowed_upstreams)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_policy_matches_exact() {
        let policy = NamespacePolicy::new("@acme");
        assert!(policy.matches("@acme"));
        assert!(policy.matches("@acme/pkg"));
        assert!(!policy.matches("@other"));
    }

    #[test]
    fn test_namespace_policy_local_only() {
        let policy = NamespacePolicy::local_only("@private");
        assert!(policy.is_local_only());
        assert!(!policy.should_delegate());
        assert!(policy.matches("@private/pkg"));
    }

    #[test]
    fn test_namespace_policy_delegate() {
        let policy = NamespacePolicy::new("@acme").delegate_to_upstream(true);
        assert!(!policy.is_local_only());
        assert!(policy.should_delegate());
    }

    #[test]
    fn test_namespace_policy_allowed_upstreams() {
        let policy = NamespacePolicy::new("@acme")
            .allowed_upstreams(vec!["public".to_string(), "partner".to_string()]);
        assert!(policy.matches_upstream("public"));
        assert!(policy.matches_upstream("partner"));
        assert!(!policy.matches_upstream("npm"));
    }

    #[test]
    fn test_namespace_policy_config() {
        let mut config = NamespacePolicyConfig::new();
        config.add_policy(NamespacePolicy::local_only("@private"));
        config.add_policy(NamespacePolicy::new("@acme").allowed_upstreams(vec!["public".to_string()]));

        assert!(config.is_local_only("@private"));
        assert!(config.is_local_only("@private/pkg"));
        assert!(!config.is_local_only("@acme"));
        assert!(!config.is_local_only("@other"));

        assert!(config.should_delegate("@acme"));
        assert!(!config.should_delegate("@private"));
        assert!(config.should_delegate("@other"));

        assert_eq!(config.allowed_upstreams("@acme"), Some(&vec!["public".to_string()]));
        assert_eq!(config.allowed_upstreams("@private"), Some(&vec![]));
    }
}