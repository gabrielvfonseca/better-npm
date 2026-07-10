//! Lockfile format parsing, generation, and resolution tracking.
//!
//! The lockfile records the exact resolved versions and integrity
//! hashes for all dependencies, ensuring reproducible installs.

use std::collections::{BTreeMap, BTreeSet};

/// Error type for lockfile operations.
#[derive(Debug, thiserror::Error)]
pub enum LockfileError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("serialization error: {0}")]
    Serialize(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid entry: {0}")]
    InvalidEntry(String),
    #[error("version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
}

/// Current lockfile version.
pub const LOCKFILE_VERSION: u32 = 1;

/// A single entry in the lockfile tracking a resolved dependency.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LockfileEntry {
    /// The exact resolved version.
    pub version: String,
    /// URL where the package was resolved from.
    pub resolved: Option<String>,
    /// Integrity hash (SRI format).
    pub integrity: Option<String>,
    /// Dependencies of this entry (name → version).
    pub dependencies: BTreeMap<String, String>,
    /// Dev dependencies of this entry.
    pub dev_dependencies: BTreeMap<String, String>,
    /// Optional dependencies of this entry.
    pub optional_dependencies: BTreeMap<String, String>,
    /// Whether this package is a dev dependency of the root.
    #[serde(default)]
    pub dev: bool,
    /// Whether this package is an optional dependency of the root.
    #[serde(default)]
    pub optional: bool,
}

impl LockfileEntry {
    /// Create a new lockfile entry.
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            resolved: None,
            integrity: None,
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            optional_dependencies: BTreeMap::new(),
            dev: false,
            optional: false,
        }
    }

    /// Set the resolved URL.
    pub fn with_resolved(mut self, url: impl Into<String>) -> Self {
        self.resolved = Some(url.into());
        self
    }

    /// Set the integrity hash.
    pub fn with_integrity(mut self, integrity: impl Into<String>) -> Self {
        self.integrity = Some(integrity.into());
        self
    }

    /// Add a dependency.
    pub fn add_dependency(&mut self, name: impl Into<String>, version: impl Into<String>) {
        self.dependencies.insert(name.into(), version.into());
    }
}

/// A lockfile containing all resolved dependency metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Lockfile {
    /// Lockfile format version.
    pub version: u32,
    /// Registry URL used for resolution.
    pub registry: Option<String>,
    /// All resolved entries, keyed by package name.
    pub entries: BTreeMap<String, LockfileEntry>,
}

impl Lockfile {
    /// Create a new empty lockfile.
    pub fn new() -> Self {
        Self {
            version: LOCKFILE_VERSION,
            registry: None,
            entries: BTreeMap::new(),
        }
    }

    /// Parse a lockfile from a JSON string.
    pub fn parse(content: &str) -> Result<Self, LockfileError> {
        let parsed: serde_json::Value =
            serde_json::from_str(content).map_err(|e| LockfileError::Parse(e.to_string()))?;

        // Detect format (json or yaml)
        let lockfile: Lockfile = serde_json::from_value(parsed.clone())
            .map_err(|e| LockfileError::Parse(e.to_string()))?;

        if lockfile.version != LOCKFILE_VERSION {
            return Err(LockfileError::VersionMismatch {
                expected: LOCKFILE_VERSION,
                actual: lockfile.version,
            });
        }

        Ok(lockfile)
    }

    /// Parse a lockfile from a YAML string.
    pub fn parse_yaml(content: &str) -> Result<Self, LockfileError> {
        let lockfile: Lockfile =
            serde_yaml::from_str(content).map_err(|e| LockfileError::Parse(e.to_string()))?;

        if lockfile.version != LOCKFILE_VERSION {
            return Err(LockfileError::VersionMismatch {
                expected: LOCKFILE_VERSION,
                actual: lockfile.version,
            });
        }

        Ok(lockfile)
    }

    /// Parse a lockfile from a TOML string.
    pub fn parse_toml(content: &str) -> Result<Self, LockfileError> {
        let lockfile: Lockfile =
            toml::from_str(content).map_err(|e| LockfileError::Parse(e.to_string()))?;

        if lockfile.version != LOCKFILE_VERSION {
            return Err(LockfileError::VersionMismatch {
                expected: LOCKFILE_VERSION,
                actual: lockfile.version,
            });
        }

        Ok(lockfile)
    }

    /// Serialize the lockfile to a JSON string.
    pub fn serialize(&self) -> Result<String, LockfileError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| LockfileError::Serialize(e.to_string()))
    }

    /// Serialize the lockfile to a YAML string.
    pub fn serialize_yaml(&self) -> Result<String, LockfileError> {
        serde_yaml::to_string(self).map_err(|e| LockfileError::Serialize(e.to_string()))
    }

    /// Serialize the lockfile to a TOML string.
    pub fn serialize_toml(&self) -> Result<String, LockfileError> {
        toml::to_string_pretty(self).map_err(|e| LockfileError::Serialize(e.to_string()))
    }

    /// Get an entry by package name.
    pub fn get(&self, name: &str) -> Option<&LockfileEntry> {
        self.entries.get(name)
    }

    /// Insert or update an entry.
    pub fn insert(&mut self, name: impl Into<String>, entry: LockfileEntry) {
        self.entries.insert(name.into(), entry);
    }

    /// Remove an entry.
    pub fn remove(&mut self, name: &str) -> Option<LockfileEntry> {
        self.entries.remove(name)
    }

    /// Check if an entry exists.
    pub fn has(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    /// Return the number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the lockfile is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Merge another lockfile into this one. Entries from `other` override.
    pub fn merge(&mut self, other: &Lockfile) {
        for (name, entry) in &other.entries {
            self.entries.insert(name.clone(), entry.clone());
        }
    }

    /// Compute the diff between this lockfile and another.
    pub fn diff(&self, other: &Lockfile) -> LockfileDiff {
        let mut added = BTreeMap::new();
        let mut removed = BTreeMap::new();
        let mut changed = BTreeMap::new();

        let self_names: BTreeSet<_> = self.entries.keys().collect();
        let other_names: BTreeSet<_> = other.entries.keys().collect();

        for name in other_names.difference(&self_names) {
            if let Some(entry) = other.entries.get(*name) {
                added.insert((*name).clone(), entry.clone());
            }
        }

        for name in self_names.difference(&other_names) {
            if let Some(entry) = self.entries.get(*name) {
                removed.insert((*name).clone(), entry.clone());
            }
        }

        for name in self_names.intersection(&other_names) {
            let self_entry = &self.entries[*name];
            let other_entry = &other.entries[*name];
            if self_entry.version != other_entry.version
                || self_entry.integrity != other_entry.integrity
            {
                changed.insert((*name).clone(), (self_entry.clone(), other_entry.clone()));
            }
        }

        LockfileDiff {
            added,
            removed,
            changed,
        }
    }

    /// Write the lockfile to a path in JSON format.
    pub fn write_to_file(&self, path: &std::path::Path) -> Result<(), LockfileError> {
        let content = self.serialize()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load a lockfile from a path (auto-detects format by extension).
    pub fn load(path: &std::path::Path) -> Result<Self, LockfileError> {
        let content = std::fs::read_to_string(path)?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("json");

        match ext {
            "yaml" | "yml" => Self::parse_yaml(&content),
            "toml" => Self::parse_toml(&content),
            _ => Self::parse(&content),
        }
    }

    /// Return an iterator over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &LockfileEntry)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Recompute the integrity hash for the lockfile itself.
    pub fn fingerprint(&self) -> Result<String, LockfileError> {
        let serialized = self.serialize()?;
        let hash = dash_hashing::hash_sha256(serialized.as_bytes());
        Ok(hex::encode(hash))
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents the difference between two lockfiles.
#[derive(Debug, Clone)]
pub struct LockfileDiff {
    /// Packages added in the new lockfile.
    pub added: BTreeMap<String, LockfileEntry>,
    /// Packages removed from the old lockfile.
    pub removed: BTreeMap<String, LockfileEntry>,
    /// Packages whose version or integrity changed.
    pub changed: BTreeMap<String, (LockfileEntry, LockfileEntry)>,
}

impl LockfileDiff {
    /// Check if there are any differences.
    pub fn has_changes(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty() || !self.changed.is_empty()
    }

    /// Number of added packages.
    pub fn added_count(&self) -> usize {
        self.added.len()
    }

    /// Number of removed packages.
    pub fn removed_count(&self) -> usize {
        self.removed.len()
    }

    /// Number of changed packages.
    pub fn changed_count(&self) -> usize {
        self.changed.len()
    }

    /// Total number of changes.
    pub fn total_changes(&self) -> usize {
        self.added_count() + self.removed_count() + self.changed_count()
    }
}

/// Find the lockfile in a directory.
pub fn find_lockfile(path: &std::path::Path) -> Option<std::path::PathBuf> {
    for name in &["dash.lock", "dash-lock.json", "package-lock.json", "dash-lock.yaml"] {
        let candidate = path.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_new() {
        let lockfile = Lockfile::new();
        assert_eq!(lockfile.version, LOCKFILE_VERSION);
        assert!(lockfile.is_empty());
    }

    #[test]
    fn test_lockfile_insert_and_get() {
        let mut lockfile = Lockfile::new();
        let entry = LockfileEntry::new("1.0.0")
            .with_resolved("https://registry.example.com/pkg-1.0.0.tar.gz")
            .with_integrity("sha512-abc123");

        lockfile.insert("test-pkg", entry);
        let retrieved = lockfile.get("test-pkg").unwrap();
        assert_eq!(retrieved.version, "1.0.0");
        assert_eq!(retrieved.resolved.as_deref(), Some("https://registry.example.com/pkg-1.0.0.tar.gz"));
    }

    #[test]
    fn test_lockfile_serialize_deserialize() {
        let mut lockfile = Lockfile::new();
        lockfile.insert("pkg-a", LockfileEntry::new("1.0.0"));
        lockfile.insert("pkg-b", LockfileEntry::new("2.0.0").with_integrity("sha256-xyz"));

        let serialized = lockfile.serialize().unwrap();
        let deserialized = Lockfile::parse(&serialized).unwrap();
        assert_eq!(deserialized.len(), 2);
        assert!(deserialized.has("pkg-a"));
        assert!(deserialized.has("pkg-b"));
    }

    #[test]
    fn test_lockfile_merge() {
        let mut lockfile1 = Lockfile::new();
        lockfile1.insert("pkg-a", LockfileEntry::new("1.0.0"));

        let mut lockfile2 = Lockfile::new();
        lockfile2.insert("pkg-b", LockfileEntry::new("2.0.0"));

        lockfile1.merge(&lockfile2);
        assert_eq!(lockfile1.len(), 2);
        assert!(lockfile1.has("pkg-b"));
    }

    #[test]
    fn test_lockfile_diff() {
        let mut old = Lockfile::new();
        old.insert("pkg-a", LockfileEntry::new("1.0.0"));
        old.insert("pkg-b", LockfileEntry::new("1.0.0"));

        let mut new = Lockfile::new();
        new.insert("pkg-a", LockfileEntry::new("2.0.0")); // changed
        new.insert("pkg-c", LockfileEntry::new("1.0.0")); // added

        let diff = old.diff(&new);
        assert!(diff.has_changes());
        assert_eq!(diff.changed_count(), 1);
        assert_eq!(diff.added_count(), 1);
        assert_eq!(diff.removed_count(), 1);
    }

    #[test]
    fn test_lockfile_fingerprint() {
        let mut lockfile = Lockfile::new();
        lockfile.insert("pkg", LockfileEntry::new("1.0.0"));
        let fp = lockfile.fingerprint().unwrap();
        assert_eq!(fp.len(), 64); // SHA-256 hex
    }

    #[test]
    fn test_lockfile_entry_deps() {
        let mut entry = LockfileEntry::new("1.0.0");
        entry.add_dependency("lodash", "^4.17.0");
        entry.add_dependency("react", "^18.0.0");
        assert_eq!(entry.dependencies.len(), 2);
    }

    #[test]
    fn test_lockfile_iter() {
        let mut lockfile = Lockfile::new();
        lockfile.insert("a", LockfileEntry::new("1.0.0"));
        lockfile.insert("b", LockfileEntry::new("2.0.0"));
        let names: Vec<_> = lockfile.iter().map(|(n, _)| n).collect();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
version: 1
entries:
  test-pkg:
    version: "1.0.0"
"#;
        let lockfile = Lockfile::parse_yaml(yaml).unwrap();
        assert!(lockfile.has("test-pkg"));
    }

    #[test]
    fn test_diff_no_changes() {
        let lockfile = Lockfile::new();
        let diff = lockfile.diff(&Lockfile::new());
        assert!(!diff.has_changes());
    }
}
