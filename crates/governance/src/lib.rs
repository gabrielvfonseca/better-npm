use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    Read,
    Resolve,
    Publish,
    Yank,
    Tag,
    Admin,
    Delegate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Scope {
    All,
    Namespace(String),
    Package(String),
    Version(String),
}

impl Scope {
    pub fn matches(&self, target: &Scope) -> bool {
        match (self, target) {
            (Scope::All, _) => true,
            (Scope::Namespace(ns), Scope::Namespace(target_ns)) => ns == target_ns,
            (Scope::Namespace(ns), Scope::Package(pkg)) => pkg.starts_with(&format!("{ns}/")),
            (Scope::Package(pkg), Scope::Package(target_pkg)) => pkg == target_pkg,
            (Scope::Package(pkg), Scope::Version(ver)) => ver.starts_with(pkg),
            (Scope::Version(ver), Scope::Version(target_ver)) => ver == target_ver,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclEntry {
    pub identity: String,
    pub permissions: Vec<Permission>,
    pub scope: Scope,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityAcl {
    pub entries: Vec<AclEntry>,
}

pub fn check_permission(
    acl: &CapabilityAcl,
    identity: &str,
    permission: &Permission,
    scope: &Scope,
) -> bool {
    acl.entries.iter().any(|entry| {
        entry.identity == identity
            || entry.identity == "did:dash:any"
            || (entry.identity.starts_with("role:") && identity.contains(&entry.identity[5..]))
            || (entry.identity.starts_with("team:") && identity.contains(&entry.identity[5..]))
    }) && acl.entries.iter().any(|entry| {
        (entry.identity == identity
            || entry.identity == "did:dash:any"
            || (entry.identity.starts_with("role:") && identity.contains(&entry.identity[5..]))
            || (entry.identity.starts_with("team:") && identity.contains(&entry.identity[5..])))
            && entry.permissions.contains(permission)
            && entry.scope.matches(scope)
    })
}

pub fn builtin_role_permissions(role: &str) -> Option<Vec<Permission>> {
    match role.to_lowercase().as_str() {
        "reader" => Some(vec![Permission::Read, Permission::Resolve]),
        "contributor" => Some(vec![Permission::Read, Permission::Resolve, Permission::Publish]),
        "maintainer" => Some(vec![
            Permission::Read,
            Permission::Resolve,
            Permission::Publish,
            Permission::Yank,
            Permission::Tag,
        ]),
        "admin" => Some(vec![
            Permission::Read,
            Permission::Resolve,
            Permission::Publish,
            Permission::Yank,
            Permission::Tag,
            Permission::Admin,
            Permission::Delegate,
        ]),
        "agent" => Some(vec![Permission::Read, Permission::Resolve]),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<Permission>,
}

impl Role {
    pub fn new(name: &str, permissions: Vec<Permission>) -> Self {
        Self { name: name.to_string(), permissions }
    }

    pub fn builtin(name: &str) -> Option<Self> {
        builtin_role_permissions(name).map(|p| Self { name: name.to_string(), permissions: p })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub identity: String,
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Org {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub owner: String,
    pub teams: Vec<Team>,
    pub members: Vec<Member>,
}

impl Org {
    pub fn new(id: &str, name: &str, owner: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            created_at: Utc::now(),
            owner: owner.to_string(),
            teams: Vec::new(),
            members: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    Publish,
    Yank,
    Grant,
    Revoke,
    PolicyChange,
    MemberAdd,
    MemberRemove,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Publish => write!(f, "publish"),
            Action::Yank => write!(f, "yank"),
            Action::Grant => write!(f, "grant"),
            Action::Revoke => write!(f, "revoke"),
            Action::PolicyChange => write!(f, "policy_change"),
            Action::MemberAdd => write!(f, "member_add"),
            Action::MemberRemove => write!(f, "member_remove"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: Action,
    pub target: String,
    pub details: String,
}

impl AuditEvent {
    pub fn new(actor: &str, action: Action, target: &str, details: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            actor: actor.to_string(),
            action,
            target: target.to_string(),
            details: details.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuditLog {
    pub events: Vec<AuditEvent>,
}

impl AuditLog {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn log_event(&mut self, event: AuditEvent) {
        self.events.push(event);
    }

    pub fn query_events(&self, filter: &AuditFilter) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| {
                if let Some(ref actor) = filter.actor {
                    if &e.actor != actor {
                        return false;
                    }
                }
                if let Some(ref action) = filter.action {
                    if e.action != *action {
                        return false;
                    }
                }
                if let Some(ref target) = filter.target {
                    if &e.target != target {
                        return false;
                    }
                }
                if let Some(ref since) = filter.since {
                    if e.timestamp < *since {
                        return false;
                    }
                }
                if let Some(ref until) = filter.until {
                    if e.timestamp > *until {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub actor: Option<String>,
    pub action: Option<Action>,
    pub target: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

pub trait StoreGovernance {
    fn get_org(&self, id: &str) -> Option<&Org>;
    fn save_org(&mut self, org: Org);
    fn get_acl(&self) -> &CapabilityAcl;
    fn save_acl(&mut self, acl: CapabilityAcl);
    fn get_audit_log(&self) -> &AuditLog;
    fn log_audit_event(&mut self, event: AuditEvent);
    fn query_audit_events(&self, filter: &AuditFilter) -> Vec<&AuditEvent>;
}

#[derive(Debug, Default)]
pub struct InMemoryGovernanceStore {
    pub org: Option<Org>,
    pub acl: CapabilityAcl,
    pub audit_log: AuditLog,
}

impl InMemoryGovernanceStore {
    pub fn new() -> Self {
        Self { org: None, acl: CapabilityAcl::default(), audit_log: AuditLog::new() }
    }
}

impl StoreGovernance for InMemoryGovernanceStore {
    fn get_org(&self, id: &str) -> Option<&Org> {
        self.org.as_ref().filter(|o| o.id == id)
    }

    fn save_org(&mut self, org: Org) {
        self.org = Some(org);
    }

    fn get_acl(&self) -> &CapabilityAcl {
        &self.acl
    }

    fn save_acl(&mut self, acl: CapabilityAcl) {
        self.acl = acl;
    }

    fn get_audit_log(&self) -> &AuditLog {
        &self.audit_log
    }

    fn log_audit_event(&mut self, event: AuditEvent) {
        self.audit_log.log_event(event);
    }

    fn query_audit_events(&self, filter: &AuditFilter) -> Vec<&AuditEvent> {
        self.audit_log.query_events(filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_permission_allows() {
        let acl = CapabilityAcl {
            entries: vec![AclEntry {
                identity: "did:dash:user:alice".into(),
                permissions: vec![Permission::Read, Permission::Publish],
                scope: Scope::Namespace("@acme".into()),
            }],
        };
        assert!(check_permission(
            &acl,
            "did:dash:user:alice",
            &Permission::Read,
            &Scope::Namespace("@acme".into())
        ));
        assert!(check_permission(
            &acl,
            "did:dash:user:alice",
            &Permission::Publish,
            &Scope::Package("@acme/http-client".into())
        ));
    }

    #[test]
    fn test_check_permission_denies_wrong_permission() {
        let acl = CapabilityAcl {
            entries: vec![AclEntry {
                identity: "did:dash:user:alice".into(),
                permissions: vec![Permission::Read],
                scope: Scope::Namespace("@acme".into()),
            }],
        };
        assert!(!check_permission(
            &acl,
            "did:dash:user:alice",
            &Permission::Admin,
            &Scope::Namespace("@acme".into())
        ));
    }

    #[test]
    fn test_check_permission_denies_unknown_identity() {
        let acl = CapabilityAcl {
            entries: vec![AclEntry {
                identity: "did:dash:user:alice".into(),
                permissions: vec![Permission::Read],
                scope: Scope::All,
            }],
        };
        assert!(!check_permission(&acl, "did:dash:user:bob", &Permission::Read, &Scope::All));
    }

    #[test]
    fn test_check_permission_scope_mismatch() {
        let acl = CapabilityAcl {
            entries: vec![AclEntry {
                identity: "did:dash:user:alice".into(),
                permissions: vec![Permission::Publish],
                scope: Scope::Package("@acme/foo".into()),
            }],
        };
        assert!(!check_permission(
            &acl,
            "did:dash:user:alice",
            &Permission::Publish,
            &Scope::Package("@acme/bar".into())
        ));
    }

    #[test]
    fn test_role_hierarchy_reader() {
        let perms = builtin_role_permissions("reader").unwrap();
        assert!(perms.contains(&Permission::Read));
        assert!(perms.contains(&Permission::Resolve));
        assert!(!perms.contains(&Permission::Publish));
    }

    #[test]
    fn test_role_hierarchy_maintainer() {
        let perms = builtin_role_permissions("maintainer").unwrap();
        assert!(perms.contains(&Permission::Publish));
        assert!(perms.contains(&Permission::Yank));
        assert!(perms.contains(&Permission::Tag));
        assert!(!perms.contains(&Permission::Admin));
    }

    #[test]
    fn test_role_hierarchy_admin() {
        let perms = builtin_role_permissions("admin").unwrap();
        assert!(perms.contains(&Permission::Admin));
        assert!(perms.contains(&Permission::Delegate));
        assert!(perms.contains(&Permission::Read));
        assert!(perms.contains(&Permission::Publish));
    }

    #[test]
    fn test_audit_log_event() {
        let mut log = AuditLog::new();
        let event = AuditEvent::new(
            "did:dash:user:alice",
            Action::Publish,
            "@acme/pkg@1.0.0",
            "published version 1.0.0",
        );
        log.log_event(event);
        assert_eq!(log.events.len(), 1);
        assert_eq!(log.events[0].actor, "did:dash:user:alice");
    }

    #[test]
    fn test_audit_filter_by_actor() {
        let mut log = AuditLog::new();
        log.log_event(AuditEvent::new("alice", Action::Publish, "pkg-a", ""));
        log.log_event(AuditEvent::new("bob", Action::Yank, "pkg-b", ""));
        log.log_event(AuditEvent::new("alice", Action::Grant, "pkg-c", ""));

        let filter = AuditFilter { actor: Some("alice".into()), ..Default::default() };
        let results = log.query_events(&filter);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.actor == "alice"));
    }

    #[test]
    fn test_audit_filter_by_action() {
        let mut log = AuditLog::new();
        log.log_event(AuditEvent::new("alice", Action::Publish, "pkg-a", ""));
        log.log_event(AuditEvent::new("bob", Action::Yank, "pkg-b", ""));
        log.log_event(AuditEvent::new("charlie", Action::Publish, "pkg-c", ""));

        let filter = AuditFilter { action: Some(Action::Publish), ..Default::default() };
        let results = log.query_events(&filter);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.action == Action::Publish));
    }

    #[test]
    fn test_audit_filter_combined() {
        let mut log = AuditLog::new();
        log.log_event(AuditEvent::new("alice", Action::Publish, "pkg-a", ""));
        log.log_event(AuditEvent::new("alice", Action::Yank, "pkg-b", ""));
        log.log_event(AuditEvent::new("bob", Action::Publish, "pkg-c", ""));

        let filter = AuditFilter {
            actor: Some("alice".into()),
            action: Some(Action::Publish),
            ..Default::default()
        };
        let results = log.query_events(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].target, "pkg-a");
    }

    #[test]
    fn test_in_memory_store() {
        let mut store = InMemoryGovernanceStore::new();
        let org = Org::new("did:dash:org:acme", "Acme Corp", "did:dash:user:alice");
        store.save_org(org);
        assert!(store.get_org("did:dash:org:acme").is_some());
        assert!(store.get_org("nonexistent").is_none());
    }

    #[test]
    fn test_in_memory_store_acl() {
        let mut store = InMemoryGovernanceStore::new();
        let acl = CapabilityAcl {
            entries: vec![AclEntry {
                identity: "did:dash:user:alice".into(),
                permissions: vec![Permission::Admin],
                scope: Scope::All,
            }],
        };
        store.save_acl(acl);
        assert!(!store.get_acl().entries.is_empty());
    }

    #[test]
    fn test_scope_matches_all() {
        assert!(Scope::All.matches(&Scope::Namespace("@acme".into())));
        assert!(Scope::All.matches(&Scope::Package("@acme/foo".into())));
        assert!(Scope::All.matches(&Scope::All));
    }

    #[test]
    fn test_scope_namespace_matches_package() {
        assert!(Scope::Namespace("@acme".into()).matches(&Scope::Package("@acme/foo".into())));
        assert!(!Scope::Namespace("@acme".into()).matches(&Scope::Package("@other/foo".into())));
    }

    #[test]
    fn test_builtin_role_reader() {
        let role = Role::builtin("reader").unwrap();
        assert_eq!(role.name, "reader");
        assert_eq!(role.permissions.len(), 2);
    }

    #[test]
    fn test_builtin_role_invalid() {
        assert!(Role::builtin("superadmin").is_none());
    }

    #[test]
    fn test_display_action() {
        assert_eq!(Action::Publish.to_string(), "publish");
        assert_eq!(Action::PolicyChange.to_string(), "policy_change");
        assert_eq!(Action::MemberAdd.to_string(), "member_add");
    }
}
