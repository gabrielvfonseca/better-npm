use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    pub digest: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaMetadata {
    pub started_on: Option<i64>,
    pub finished_on: Option<i64>,
    pub hermetic: bool,
    pub reproducible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlsaPredicate {
    pub builder_id: String,
    pub build_type: String,
    pub recipe: Value,
    pub metadata: SlsaMetadata,
    pub materials: Vec<Subject>,
    pub invocation: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    pub type_url: String,
    pub predicate_type: String,
    pub subject: Vec<Subject>,
    pub predicate: Value,
}

pub fn build_attestation(
    package_name: &str,
    version: &str,
    cid: &str,
    builder_id: &str,
) -> Attestation {
    let mut digest = BTreeMap::new();
    digest.insert("b3".to_string(), cid.to_string());

    let subject = Subject { name: format!("{package_name}-{version}"), digest };

    let predicate = serde_json::json!({
        "builder": {
            "id": builder_id
        },
        "buildType": "https://dash.dev/build/tsc@1",
        "recipe": {
            "type": "dash-build",
            "version": version
        },
        "metadata": {
            "hermetic": true,
            "reproducible": false
        },
        "materials": [],
        "invocation": {
            "configSource": {
                "uri": format!("pkg:{}@{}", package_name, version),
                "digest": { "b3": cid }
            }
        }
    });

    Attestation {
        type_url: "https://in-toto.io/Statement/v1".to_string(),
        predicate_type: "https://slsa.dev/provenance/v1".to_string(),
        subject: vec![subject],
        predicate,
    }
}

pub fn verify_attestation(attestation: &Attestation) -> bool {
    if attestation.type_url != "https://in-toto.io/Statement/v1" {
        return false;
    }
    if attestation.predicate_type != "https://slsa.dev/provenance/v1" {
        return false;
    }
    if attestation.subject.is_empty() {
        return false;
    }
    let builder = attestation.predicate.get("builder");
    if builder.is_none() {
        return false;
    }
    let build_type = attestation.predicate.get("buildType");
    if build_type.is_none() {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_attestation() {
        let att = build_attestation("@scope/pkg", "1.2.3", "b3:abc123", "dash:registry");
        assert_eq!(att.type_url, "https://in-toto.io/Statement/v1");
        assert_eq!(att.predicate_type, "https://slsa.dev/provenance/v1");
        assert_eq!(att.subject.len(), 1);
        assert_eq!(att.subject[0].name, "@scope/pkg-1.2.3");
        assert_eq!(att.subject[0].digest.get("b3").unwrap(), "b3:abc123");
    }

    #[test]
    fn test_verify_attestation() {
        let att = build_attestation("pkg", "0.1.0", "b3:def456", "dash:ci-builder");
        assert!(verify_attestation(&att));
    }

    #[test]
    fn test_verify_attestation_invalid_type() {
        let mut att = build_attestation("pkg", "1.0", "b3:xyz", "builder");
        att.type_url = "invalid".to_string();
        assert!(!verify_attestation(&att));
    }

    #[test]
    fn test_verify_attestation_missing_builder() {
        let mut att = build_attestation("pkg", "1.0", "b3:xyz", "builder");
        let pred = att.predicate.as_object_mut().unwrap();
        pred.remove("builder");
        assert!(!verify_attestation(&att));
    }

    #[test]
    fn test_verify_attestation_empty_subject() {
        let mut att = build_attestation("pkg", "1.0", "b3:xyz", "builder");
        att.subject.clear();
        assert!(!verify_attestation(&att));
    }
}
