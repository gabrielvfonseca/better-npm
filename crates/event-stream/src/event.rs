use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventType {
    PackagePublished,
    PackageYanked,
    TagMoved,
    VulnerabilityReported,
    PolicyViolation,
    MemberAdded,
    MemberRemoved,
    VersionAdded,
    ReputationChanged,
    BuildCompleted,
    ScanCompleted,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventType::PackagePublished => "package-published",
            EventType::PackageYanked => "package-yanked",
            EventType::TagMoved => "tag-moved",
            EventType::VulnerabilityReported => "vulnerability-reported",
            EventType::PolicyViolation => "policy-violation",
            EventType::MemberAdded => "member-added",
            EventType::MemberRemoved => "member-removed",
            EventType::VersionAdded => "version-added",
            EventType::ReputationChanged => "reputation-changed",
            EventType::BuildCompleted => "build-completed",
            EventType::ScanCompleted => "scan-completed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "package-published" => Some(EventType::PackagePublished),
            "package-yanked" => Some(EventType::PackageYanked),
            "tag-moved" => Some(EventType::TagMoved),
            "vulnerability-reported" => Some(EventType::VulnerabilityReported),
            "policy-violation" => Some(EventType::PolicyViolation),
            "member-added" => Some(EventType::MemberAdded),
            "member-removed" => Some(EventType::MemberRemoved),
            "version-added" => Some(EventType::VersionAdded),
            "reputation-changed" => Some(EventType::ReputationChanged),
            "build-completed" => Some(EventType::BuildCompleted),
            "scan-completed" => Some(EventType::ScanCompleted),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSource {
    pub registry: String,
    pub instance_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EventPayload {
    PackagePublished(PackagePublishedPayload),
    PackageYanked(PackageYankedPayload),
    TagMoved(TagMovedPayload),
    VulnerabilityReported(VulnerabilityReportedPayload),
    PolicyViolation(PolicyViolationPayload),
    MemberAdded(MemberAddedPayload),
    MemberRemoved(MemberRemovedPayload),
    VersionAdded(VersionAddedPayload),
    ReputationChanged(ReputationChangedPayload),
    BuildCompleted(BuildCompletedPayload),
    ScanCompleted(ScanCompletedPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagePublishedPayload {
    pub package: String,
    pub version: String,
    pub cid: String,
    pub publisher: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageYankedPayload {
    pub package: String,
    pub version: String,
    pub reason: String,
    pub yanker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagMovedPayload {
    pub package: String,
    pub tag: String,
    pub from_version: String,
    pub to_version: String,
    pub mover: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReportedPayload {
    pub cve_id: String,
    pub affected_packages: Vec<String>,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolationPayload {
    pub policy_name: String,
    pub package: String,
    pub version: String,
    pub severity: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberAddedPayload {
    pub org: String,
    pub member: String,
    pub roles: Vec<String>,
    pub added_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemberRemovedPayload {
    pub org: String,
    pub member: String,
    pub removed_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionAddedPayload {
    pub package: String,
    pub version: String,
    pub is_first_version: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationChangedPayload {
    pub package: String,
    pub old_score: f64,
    pub new_score: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCompletedPayload {
    pub package: String,
    pub version: String,
    pub builder: String,
    pub success: bool,
    pub output_cid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCompletedPayload {
    pub package: String,
    pub version: String,
    pub scanner: String,
    pub findings: Vec<ScanFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanFinding {
    pub id: String,
    pub severity: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub payload: EventPayload,
    pub signature: Option<String>,
    pub source: EventSource,
}

impl Event {
    pub fn new(event_type: EventType, payload: EventPayload, source: EventSource) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            payload,
            signature: None,
            source,
        }
    }

    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn event_type_str(&self) -> &'static str {
        self.event_type.as_str()
    }
}

impl EventPayload {
    pub fn event_type(&self) -> EventType {
        match self {
            EventPayload::PackagePublished(_) => EventType::PackagePublished,
            EventPayload::PackageYanked(_) => EventType::PackageYanked,
            EventPayload::TagMoved(_) => EventType::TagMoved,
            EventPayload::VulnerabilityReported(_) => EventType::VulnerabilityReported,
            EventPayload::PolicyViolation(_) => EventType::PolicyViolation,
            EventPayload::MemberAdded(_) => EventType::MemberAdded,
            EventPayload::MemberRemoved(_) => EventType::MemberRemoved,
            EventPayload::VersionAdded(_) => EventType::VersionAdded,
            EventPayload::ReputationChanged(_) => EventType::ReputationChanged,
            EventPayload::BuildCompleted(_) => EventType::BuildCompleted,
            EventPayload::ScanCompleted(_) => EventType::ScanCompleted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_string_conversion() {
        assert_eq!(EventType::PackagePublished.as_str(), "package-published");
        assert_eq!(EventType::from_str("package-published"), Some(EventType::PackagePublished));
        assert_eq!(EventType::from_str("unknown"), None);
    }

    #[test]
    fn test_event_creation() {
        let source = EventSource {
            registry: "test".to_string(),
            instance_id: "test-1".to_string(),
        };
        let payload = EventPayload::PackagePublished(PackagePublishedPayload {
            package: "test".to_string(),
            version: "1.0.0".to_string(),
            cid: "b3:abc".to_string(),
            publisher: "did:bnpm:test".to_string(),
            namespace: "test".to_string(),
        });
        let event = Event::new(EventType::PackagePublished, payload, source);
        assert!(event.id != Uuid::nil());
        assert_eq!(event.event_type, EventType::PackagePublished);
    }

    #[test]
    fn test_event_payload_event_type() {
        let payload = EventPayload::PackagePublished(PackagePublishedPayload {
            package: "test".to_string(),
            version: "1.0.0".to_string(),
            cid: "b3:abc".to_string(),
            publisher: "did:bnpm:test".to_string(),
            namespace: "test".to_string(),
        });
        assert_eq!(payload.event_type(), EventType::PackagePublished);
    }
}