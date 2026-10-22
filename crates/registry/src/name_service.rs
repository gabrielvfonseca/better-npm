use std::sync::Arc;

use dashmap::DashMap;
use semver::Version;
use serde::{Deserialize, Serialize};

use bnpm_provenance::InclusionProof;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    pub version: String,
    pub cid: String,
    pub log_index: u64,
    pub inclusion_proof: Option<InclusionProof>,
    pub provenance: Option<bnpm_provenance::Attestation>,
}

#[derive(Clone)]
pub struct NameService {
    bindings: Arc<DashMap<String, Vec<Binding>>>,
    tags: Arc<DashMap<String, DashMap<String, String>>>,
}

impl NameService {
    pub fn new() -> Self {
        Self { bindings: Arc::new(DashMap::new()), tags: Arc::new(DashMap::new()) }
    }

    pub fn register(
        &self,
        name: &str,
        version: &str,
        cid: &str,
        log_index: u64,
        inclusion_proof: Option<InclusionProof>,
        provenance: Option<bnpm_provenance::Attestation>,
    ) -> Result<(), NameServiceError> {
        let binding = Binding {
            version: version.to_string(),
            cid: cid.to_string(),
            log_index,
            inclusion_proof,
            provenance,
        };

        let mut entries = self.bindings.entry(name.to_string()).or_default();
        if entries.iter().any(|b| b.version == version) {
            return Err(NameServiceError::VersionExists(name.to_string(), version.to_string()));
        }
        entries.push(binding);
        Ok(())
    }

    pub fn resolve(&self, name: &str, ranges: &[VersionRange]) -> Vec<Binding> {
        let entries = match self.bindings.get(name) {
            Some(e) => e,
            None => return Vec::new(),
        };

        if ranges.is_empty() {
            return entries.clone();
        }

        let mut results = Vec::new();
        for binding in entries.iter() {
            let ver = match Version::parse(&binding.version) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if ranges.iter().any(|r| r.matches(&ver)) {
                results.push(binding.clone());
            }
        }
        results
    }

    pub fn resolve_exact(&self, name: &str, version: &str) -> Option<Binding> {
        self.bindings
            .get(name)
            .and_then(|entries| entries.iter().find(|b| b.version == version).cloned())
    }

    pub fn tag(&self, name: &str, tag: &str, version: &str) -> Result<(), NameServiceError> {
        let tag_map = self.tags.entry(name.to_string()).or_default();
        tag_map.insert(tag.to_string(), version.to_string());
        Ok(())
    }

    pub fn resolve_tag(&self, name: &str, tag: &str) -> Option<String> {
        self.tags.get(name).and_then(|tag_map| tag_map.get(tag).map(|v| v.clone()))
    }

    pub fn has_name(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    pub fn list_versions(&self, name: &str) -> Vec<String> {
        self.bindings
            .get(name)
            .map(|entries| entries.iter().map(|b| b.version.clone()).collect())
            .unwrap_or_default()
    }
}

impl Default for NameService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub enum VersionRange {
    Exact(String),
    Constraint(String, String, bool, bool),
}

impl VersionRange {
    pub fn matches(&self, ver: &Version) -> bool {
        match self {
            VersionRange::Exact(exact) => Version::parse(exact).map(|e| e == *ver).unwrap_or(false),
            VersionRange::Constraint(low, high, low_inc, high_inc) => {
                let low_ver = match Version::parse(low) {
                    Ok(v) => v,
                    Err(_) => return false,
                };
                let low_ok = if *low_inc { *ver >= low_ver } else { *ver > low_ver };
                if !low_ok {
                    return false;
                }
                if !high.is_empty() {
                    let high_ver = match Version::parse(high) {
                        Ok(v) => v,
                        Err(_) => return false,
                    };
                    let high_ok = if *high_inc { *ver <= high_ver } else { *ver < high_ver };
                    if !high_ok {
                        return false;
                    }
                }
                true
            },
        }
    }
}

impl From<bnpm_proto::dpr_v1::VersionRange> for VersionRange {
    fn from(vr: bnpm_proto::dpr_v1::VersionRange) -> Self {
        use bnpm_proto::dpr_v1::version_range::Range;
        match vr.range {
            Some(Range::Exact(v)) => VersionRange::Exact(v),
            Some(Range::Constraint(c)) => {
                VersionRange::Constraint(c.low, c.high, c.low_inclusive, c.high_inclusive)
            },
            None => VersionRange::Exact(String::new()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NameServiceError {
    #[error("version already exists: {0}@{1}")]
    VersionExists(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_resolve_exact() {
        let ns = NameService::new();
        ns.register("pkg-a", "1.0.0", "b3:abc", 0, None, None).unwrap();
        let binding = ns.resolve_exact("pkg-a", "1.0.0").unwrap();
        assert_eq!(binding.version, "1.0.0");
        assert_eq!(binding.cid, "b3:abc");
    }

    #[test]
    fn test_register_duplicate_version_fails() {
        let ns = NameService::new();
        ns.register("pkg", "1.0.0", "b3:a", 0, None, None).unwrap();
        let err = ns.register("pkg", "1.0.0", "b3:b", 1, None, None);
        assert!(err.is_err());
    }

    #[test]
    fn test_resolve_with_ranges() {
        let ns = NameService::new();
        ns.register("pkg", "1.0.0", "b3:a", 0, None, None).unwrap();
        ns.register("pkg", "1.5.0", "b3:b", 1, None, None).unwrap();
        ns.register("pkg", "2.0.0", "b3:c", 2, None, None).unwrap();

        let ranges =
            vec![VersionRange::Constraint("1.0.0".to_string(), "2.0.0".to_string(), true, false)];
        let results = ns.resolve("pkg", &ranges);
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|b| b.version == "1.0.0"));
        assert!(results.iter().any(|b| b.version == "1.5.0"));
    }

    #[test]
    fn test_resolve_nonexistent_name() {
        let ns = NameService::new();
        let results = ns.resolve("nonexistent", &[]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_tag_and_resolve_tag() {
        let ns = NameService::new();
        ns.register("pkg", "1.0.0", "b3:a", 0, None, None).unwrap();
        ns.tag("pkg", "latest", "1.0.0").unwrap();
        assert_eq!(ns.resolve_tag("pkg", "latest"), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_version_range_matches() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("2.0.0").unwrap();
        let range = VersionRange::Constraint("1.0.0".to_string(), "2.0.0".to_string(), true, false);
        assert!(range.matches(&v1));
        assert!(!range.matches(&v2));

        let exact = VersionRange::Exact("1.0.0".to_string());
        assert!(exact.matches(&v1));
        assert!(!exact.matches(&v2));
    }
}
