use dash_lockfile::{CapValue, LockCapabilities};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CapabilitySet {
    pub net: Vec<String>,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub env: Vec<String>,
    pub subprocess: bool,
}

impl From<&LockCapabilities> for CapabilitySet {
    fn from(lc: &LockCapabilities) -> Self {
        Self {
            net: cap_value_to_vec(&lc.net),
            fs_read: cap_value_to_vec(&lc.fs_read),
            fs_write: cap_value_to_vec(&lc.fs_write),
            env: cap_value_to_vec(&lc.env),
            subprocess: lc.subprocess.unwrap_or(false),
        }
    }
}

fn cap_value_to_vec(cv: &Option<CapValue>) -> Vec<String> {
    match cv {
        Some(CapValue::Allowlist(items)) => items.clone(),
        _ => Vec::new(),
    }
}

impl CapabilitySet {
    pub fn new(
        net: Vec<String>,
        fs_read: Vec<String>,
        fs_write: Vec<String>,
        env: Vec<String>,
        subprocess: bool,
    ) -> Self {
        Self { net, fs_read, fs_write, env, subprocess }
    }

    pub fn intersection(&self, policy: &CapabilitySet) -> CapabilitySet {
        CapabilitySet {
            net: intersect_lists(&self.net, &policy.net),
            fs_read: intersect_lists(&self.fs_read, &policy.fs_read),
            fs_write: intersect_lists(&self.fs_write, &policy.fs_write),
            env: intersect_lists(&self.env, &policy.env),
            subprocess: self.subprocess && policy.subprocess,
        }
    }

    pub fn diff(&self, new: &CapabilitySet) -> CapabilityDiff {
        CapabilityDiff {
            added_net: diff_added(&self.net, &new.net),
            removed_net: diff_removed(&self.net, &new.net),
            added_fs_read: diff_added(&self.fs_read, &new.fs_read),
            removed_fs_read: diff_removed(&self.fs_read, &new.fs_read),
            added_fs_write: diff_added(&self.fs_write, &new.fs_write),
            removed_fs_write: diff_removed(&self.fs_write, &new.fs_write),
            added_env: diff_added(&self.env, &new.env),
            removed_env: diff_removed(&self.env, &new.env),
            added_subprocess: !self.subprocess && new.subprocess,
            removed_subprocess: self.subprocess && !new.subprocess,
        }
    }

    pub fn is_escalation(&self) -> bool {
        !self.net.is_empty()
            || !self.fs_read.is_empty()
            || !self.fs_write.is_empty()
            || !self.env.is_empty()
            || self.subprocess
    }

    pub fn to_lock_capabilities(&self) -> LockCapabilities {
        LockCapabilities {
            net: Some(vec_to_cap_value(&self.net)),
            fs_read: Some(vec_to_cap_value(&self.fs_read)),
            fs_write: Some(vec_to_cap_value(&self.fs_write)),
            env: Some(vec_to_cap_value(&self.env)),
            subprocess: if self.subprocess { Some(true) } else { None },
        }
    }
}

fn vec_to_cap_value(items: &[String]) -> CapValue {
    if items.is_empty() { CapValue::Denied } else { CapValue::Allowlist(items.to_vec()) }
}

fn intersect_lists(a: &[String], b: &[String]) -> Vec<String> {
    a.iter().filter(|item| b.contains(item)).cloned().collect()
}

fn diff_added(old: &[String], new: &[String]) -> Vec<String> {
    new.iter().filter(|item| !old.contains(item)).cloned().collect()
}

fn diff_removed(old: &[String], new: &[String]) -> Vec<String> {
    old.iter().filter(|item| !new.contains(item)).cloned().collect()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityDiff {
    pub added_net: Vec<String>,
    pub removed_net: Vec<String>,
    pub added_fs_read: Vec<String>,
    pub removed_fs_read: Vec<String>,
    pub added_fs_write: Vec<String>,
    pub removed_fs_write: Vec<String>,
    pub added_env: Vec<String>,
    pub removed_env: Vec<String>,
    pub added_subprocess: bool,
    pub removed_subprocess: bool,
}

impl CapabilityDiff {
    pub fn has_changes(&self) -> bool {
        !self.added_net.is_empty()
            || !self.removed_net.is_empty()
            || !self.added_fs_read.is_empty()
            || !self.removed_fs_read.is_empty()
            || !self.added_fs_write.is_empty()
            || !self.removed_fs_write.is_empty()
            || !self.added_env.is_empty()
            || !self.removed_env.is_empty()
            || self.added_subprocess
            || self.removed_subprocess
    }

    pub fn is_escalation(&self) -> bool {
        !self.added_net.is_empty()
            || !self.added_fs_read.is_empty()
            || !self.added_fs_write.is_empty()
            || !self.added_env.is_empty()
            || self.added_subprocess
    }
}

pub fn generate_permissions(caps: &CapabilitySet, runtime: &str) -> Vec<String> {
    match runtime {
        "deno" => generate_deno_permissions(caps),
        "node" => generate_node_permissions(caps),
        _ => Vec::new(),
    }
}

fn generate_deno_permissions(caps: &CapabilitySet) -> Vec<String> {
    let mut flags = Vec::new();
    if !caps.net.is_empty() {
        flags.push(format!("--allow-net={}", caps.net.join(",")));
    }
    if !caps.fs_read.is_empty() {
        flags.push(format!("--allow-read={}", caps.fs_read.join(",")));
    }
    if !caps.fs_write.is_empty() {
        flags.push(format!("--allow-write={}", caps.fs_write.join(",")));
    }
    if !caps.env.is_empty() {
        flags.push(format!("--allow-env={}", caps.env.join(",")));
    }
    if caps.subprocess {
        flags.push("--allow-run".to_string());
    }
    if flags.is_empty() {
        flags.push("--no-allow".to_string());
    }
    flags
}

fn generate_node_permissions(caps: &CapabilitySet) -> Vec<String> {
    let mut flags = vec!["--experimental-permission".to_string()];
    if !caps.fs_read.is_empty() {
        flags.push(format!("--allow-fs-read={}", caps.fs_read.join(",")));
    }
    if !caps.fs_write.is_empty() {
        flags.push(format!("--allow-fs-write={}", caps.fs_write.join(",")));
    }
    if !caps.net.is_empty() {
        flags.push(format!("--allow-net={}", caps.net.join(",")));
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_caps() -> CapabilitySet {
        CapabilitySet::new(
            vec!["https://api.acme.dev".to_string()],
            vec!["./config".to_string()],
            vec![],
            vec!["DB_URL".to_string()],
            false,
        )
    }

    #[test]
    fn test_capability_intersection() {
        let requested = test_caps();
        let policy = CapabilitySet::new(
            vec!["https://api.acme.dev".to_string(), "https://other.com".to_string()],
            vec!["./config".to_string()],
            vec![],
            vec![],
            false,
        );
        let granted = requested.intersection(&policy);
        assert_eq!(granted.net, vec!["https://api.acme.dev"]);
        assert_eq!(granted.fs_read, vec!["./config"]);
        assert!(granted.env.is_empty());
    }

    #[test]
    fn test_capability_diff() {
        let old =
            CapabilitySet::new(vec!["https://old.com".to_string()], vec![], vec![], vec![], false);
        let new = CapabilitySet::new(
            vec!["https://old.com".to_string(), "https://new.com".to_string()],
            vec!["./secret".to_string()],
            vec![],
            vec![],
            true,
        );
        let diff = old.diff(&new);
        assert!(diff.has_changes());
        assert!(diff.is_escalation());
        assert!(diff.added_subprocess);
        assert_eq!(diff.added_net, vec!["https://new.com"]);
        assert_eq!(diff.added_fs_read, vec!["./secret"]);
    }

    #[test]
    fn test_capability_diff_no_change() {
        let old = test_caps();
        let diff = old.diff(&old);
        assert!(!diff.has_changes());
        assert!(!diff.is_escalation());
    }

    #[test]
    fn test_deno_permissions() {
        let caps = test_caps();
        let flags = generate_permissions(&caps, "deno");
        assert!(flags.contains(&"--allow-net=https://api.acme.dev".to_string()));
        assert!(flags.contains(&"--allow-read=./config".to_string()));
        assert!(flags.contains(&"--allow-env=DB_URL".to_string()));
    }

    #[test]
    fn test_node_permissions() {
        let caps = test_caps();
        let flags = generate_permissions(&caps, "node");
        assert!(flags.contains(&"--experimental-permission".to_string()));
        assert!(flags.contains(&"--allow-fs-read=./config".to_string()));
        assert!(flags.contains(&"--allow-net=https://api.acme.dev".to_string()));
    }

    #[test]
    fn test_worker_permissions_noop() {
        let caps = test_caps();
        let flags = generate_permissions(&caps, "worker");
        assert!(flags.is_empty());
    }

    #[test]
    fn test_is_escalation_true() {
        let caps = CapabilitySet::new(vec![], vec![], vec![], vec![], true);
        assert!(caps.is_escalation());
    }

    #[test]
    fn test_is_escalation_false() {
        let caps = CapabilitySet::new(vec![], vec![], vec![], vec![], false);
        assert!(!caps.is_escalation());
    }

    #[test]
    fn test_lock_capabilities_roundtrip() {
        let caps = test_caps();
        let lc = caps.to_lock_capabilities();
        let restored = CapabilitySet::from(&lc);
        assert_eq!(caps.net, restored.net);
        assert_eq!(caps.env, restored.env);
    }

    #[test]
    fn test_diff_correct_removed_net() {
        let old =
            CapabilitySet::new(vec!["https://old.com".to_string()], vec![], vec![], vec![], false);
        let new = CapabilitySet::new(vec![], vec![], vec![], vec![], false);
        let diff = old.diff(&new);
        assert_eq!(diff.removed_net, vec!["https://old.com"]);
        assert!(!diff.is_escalation());
    }

    #[test]
    fn test_serde_json_roundtrip() {
        let caps = test_caps();
        let json = serde_json::to_string(&caps).unwrap();
        let restored: CapabilitySet = serde_json::from_str(&json).unwrap();
        assert_eq!(caps.net, restored.net);
        assert_eq!(caps.fs_read, restored.fs_read);
        assert_eq!(caps.env, restored.env);
    }
}
