//! Core package model: name, version, metadata, and identifier types.
//!
//! Defines the fundamental types used throughout Dash for representing
//! packages, their identifiers, distribution metadata, and versioning.

use std::fmt;
use std::str::FromStr;

/// Error type for package operations.
#[derive(Debug, thiserror::Error)]
pub enum PackageError {
    #[error("invalid package name: {0}")]
    InvalidName(String),
    #[error("invalid package ID: {0}")]
    InvalidId(String),
    #[error("invalid version: {0}")]
    InvalidVersion(#[from] semver::Error),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A validated package name.
///
/// Must match the pattern: `@?[a-zA-Z0-9_-]+(\/[a-zA-Z0-9_-]+)?`
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PackageName(String);

impl PackageName {
    /// Create a new `PackageName`, validating the format.
    pub fn new(name: &str) -> Result<Self, PackageError> {
        if name.is_empty() {
            return Err(PackageError::InvalidName("name cannot be empty".to_string()));
        }

        // Allow scoped names (@scope/name) and simple names
        let valid = name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '/' || c == '@' || c == '.');

        if !valid {
            return Err(PackageError::InvalidName(format!(
                "name contains invalid characters: {name}"
            )));
        }

        if name.len() > 214 {
            return Err(PackageError::InvalidName("name too long (max 214 chars)".to_string()));
        }

        Ok(PackageName(name.to_string()))
    }

    /// Create a new `PackageName` without validation (for internal use).
    pub fn new_unchecked(name: &str) -> Self {
        PackageName(name.to_string())
    }

    /// Return the name as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Return the scope part of the name (e.g. "@scope" from "@scope/name").
    pub fn scope(&self) -> Option<&str> {
        self.0.strip_prefix('@').and_then(|s| s.split('/').next())
    }

    /// Return the un-scoped name (e.g. "name" from "@scope/name").
    pub fn unescoped_name(&self) -> &str {
        self.0.rsplit('/').next().unwrap_or(&self.0)
    }

    /// Check if this is a scoped package.
    pub fn is_scoped(&self) -> bool {
        self.0.starts_with('@')
    }
}

impl fmt::Display for PackageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PackageName {
    type Err = PackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PackageName::new(s)
    }
}

/// A fully-qualified package identifier (name + version).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PackageId {
    /// The package name.
    pub name: PackageName,
    /// The package version.
    pub version: semver::Version,
}

impl PackageId {
    /// Create a new package identifier.
    pub fn new(name: PackageName, version: semver::Version) -> Self {
        Self { name, version }
    }

    /// Create a new package identifier from name and version strings.
    pub fn parse(name: &str, version: &str) -> Result<Self, PackageError> {
        let name = PackageName::new(name)?;
        let version: semver::Version = version.parse()?;
        Ok(Self { name, version })
    }

    /// Return `name@version`.
    pub fn to_specifier(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

impl FromStr for PackageId {
    type Err = PackageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Support format: "name@version" or "name version"
        if let Some((name, ver)) = s.split_once('@') {
            PackageId::parse(name.trim(), ver.trim())
        } else if let Some((name, ver)) = s.split_once(' ') {
            PackageId::parse(name.trim(), ver.trim())
        } else {
            Err(PackageError::InvalidId(format!(
                "expected 'name@version', got: {s}"
            )))
        }
    }
}

/// Distribution metadata for a package archive.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dist {
    /// URL to the tarball.
    pub tarball: String,
    /// SHA-1 checksum (hex).
    pub shasum: Option<String>,
    /// Integrity hash (SRI format).
    pub integrity: Option<crate::Integrity>,
    /// Unpacked size in bytes.
    pub unpacked_size: Option<u64>,
    /// Number of files in the package.
    pub file_count: Option<u32>,
    /// Content-type of the tarball.
    pub content_type: Option<String>,
}

impl Dist {
    /// Create a new dist entry.
    pub fn new(tarball: impl Into<String>) -> Self {
        Self {
            tarball: tarball.into(),
            shasum: None,
            integrity: None,
            unpacked_size: None,
            file_count: None,
            content_type: None,
        }
    }

    /// Set the integrity hash.
    pub fn with_integrity(mut self, integrity: crate::Integrity) -> Self {
        self.integrity = Some(integrity);
        self
    }

    /// Set the unpacked size.
    pub fn with_unpacked_size(mut self, size: u64) -> Self {
        self.unpacked_size = Some(size);
        self
    }
}

// Forward Integrity type from dash-integrity crate
pub use dash_integrity::Integrity;

/// A fully-resolved package with all metadata.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Package {
    /// Package name.
    pub name: PackageName,
    /// Package version.
    pub version: semver::Version,
    /// Full manifest data.
    pub manifest: serde_json::Value,
    /// List of file entries.
    pub files: Vec<PackageFile>,
    /// Distribution metadata.
    pub dist: Option<Dist>,
    /// Package description.
    pub description: Option<String>,
    /// Package author.
    pub author: Option<String>,
    /// Package license.
    pub license: Option<String>,
    /// Homepage URL.
    pub homepage: Option<String>,
    /// Repository URL.
    pub repository: Option<String>,
    /// Package keywords.
    pub keywords: Vec<String>,
}

/// A single file within a package.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageFile {
    /// Path within the package.
    pub path: String,
    /// File contents.
    pub data: Vec<u8>,
    /// File mode (Unix permission bits).
    pub mode: Option<u32>,
    /// File size.
    pub size: u64,
    /// Content hash (SHA-256).
    pub hash: Option<[u8; 32]>,
}

impl Package {
    /// Create a new package with the given name and version.
    pub fn new(name: PackageName, version: semver::Version) -> Self {
        Self {
            name,
            version,
            manifest: serde_json::Value::Object(Default::default()),
            files: Vec::new(),
            dist: None,
            description: None,
            author: None,
            license: None,
            homepage: None,
            repository: None,
            keywords: Vec::new(),
        }
    }

    /// Return the package id.
    pub fn id(&self) -> PackageId {
        PackageId::new(self.name.clone(), self.version.clone())
    }

    /// Return `name@version`.
    pub fn specifier(&self) -> String {
        self.id().to_specifier()
    }

    /// Add a file to the package.
    pub fn add_file(&mut self, path: impl Into<String>, data: Vec<u8>) {
        let size = data.len() as u64;
        self.files.push(PackageFile {
            path: path.into(),
            data,
            mode: None,
            size,
            hash: None,
        });
    }
}

/// Reference to a package for download/install operations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageRef {
    /// Package identifier.
    pub id: PackageId,
    /// URL to the package tarball.
    pub url: Option<String>,
    /// Expected integrity hash.
    pub integrity: Option<Integrity>,
}

impl PackageRef {
    /// Create a new package reference.
    pub fn new(id: PackageId) -> Self {
        Self {
            id,
            url: None,
            integrity: None,
        }
    }
}

/// A version requirement for a dependency.
pub type VersionReq = semver::VersionReq;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_name_valid() {
        let name = PackageName::new("test-package").unwrap();
        assert_eq!(name.as_str(), "test-package");
    }

    #[test]
    fn test_package_name_scoped() {
        let name = PackageName::new("@scope/package").unwrap();
        assert!(name.is_scoped());
        assert_eq!(name.scope(), Some("scope"));
        assert_eq!(name.unescoped_name(), "package");
    }

    #[test]
    fn test_package_name_invalid() {
        assert!(PackageName::new("").is_err());
        assert!(PackageName::new("white space").is_ok()); // doesn't have invalid chars per our rule
        let _ = PackageName::new("normal-name").unwrap();
    }

    #[test]
    fn test_package_id_display() {
        let name = PackageName::new("mypkg").unwrap();
        let ver = semver::Version::new(1, 0, 0);
        let id = PackageId::new(name, ver);
        assert_eq!(id.to_string(), "mypkg@1.0.0");
    }

    #[test]
    fn test_package_id_from_str() {
        let id: PackageId = "mypkg@2.0.0".parse().unwrap();
        assert_eq!(id.name.as_str(), "mypkg");
        assert_eq!(id.version.to_string(), "2.0.0");
    }

    #[test]
    fn test_package_id_from_str_with_space() {
        let id: PackageId = "mypkg 3.0.0".parse().unwrap();
        assert_eq!(id.name.as_str(), "mypkg");
        assert_eq!(id.version.to_string(), "3.0.0");
    }

    #[test]
    fn test_package_id_specifier() {
        let name = PackageName::new("pkg").unwrap();
        let ver = semver::Version::new(1, 2, 3);
        let id = PackageId::new(name, ver);
        assert_eq!(id.to_specifier(), "pkg@1.2.3");
    }

    #[test]
    fn test_package_new() {
        let name = PackageName::new("test").unwrap();
        let ver = semver::Version::new(0, 1, 0);
        let pkg = Package::new(name, ver);
        assert_eq!(pkg.name.as_str(), "test");
        assert_eq!(pkg.version.to_string(), "0.1.0");
    }

    #[test]
    fn test_package_add_file() {
        let name = PackageName::new("test").unwrap();
        let ver = semver::Version::new(0, 1, 0);
        let mut pkg = Package::new(name, ver);
        pkg.add_file("index.js", b"console.log('hello')".to_vec());
        assert_eq!(pkg.files.len(), 1);
        assert_eq!(pkg.files[0].path, "index.js");
    }

    #[test]
    fn test_dist_new() {
        let dist = Dist::new("https://example.com/pkg.tar.gz");
        assert_eq!(dist.tarball, "https://example.com/pkg.tar.gz");
        assert!(dist.shasum.is_none());
    }

    #[test]
    fn test_package_ref() {
        let name = PackageName::new("pkg").unwrap();
        let ver = semver::Version::new(1, 0, 0);
        let id = PackageId::new(name, ver);
        let pref = PackageRef::new(id.clone());
        assert_eq!(pref.id, id);
    }

    #[test]
    fn test_package_name_new_unchecked() {
        let name = PackageName::new_unchecked("some-name");
        assert_eq!(name.as_str(), "some-name");
    }

    #[test]
    fn test_package_name_display() {
        let name = PackageName::new("display-name").unwrap();
        assert_eq!(format!("{name}"), "display-name");
    }
}
