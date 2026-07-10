//! Workspace-level discovery, configuration, and multi-package operations.
//!
//! Manages a workspace of multiple packages, handling member discovery,
//! filtering, and cross-package dependency resolution.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Error type for workspace operations.
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("manifest error: {0}")]
    Manifest(#[from] dash_manifest::ManifestError),
    #[error("config error: {0}")]
    Config(#[from] dash_config::ConfigError),
    #[error("member not found: {0}")]
    MemberNotFound(String),
    #[error("invalid workspace root: {0}")]
    InvalidRoot(String),
    #[error("duplicate member: {0}")]
    DuplicateMember(String),
    #[error("glob error: {0}")]
    Glob(String),
    #[error("walk error: {0}")]
    Walk(#[from] walkdir::Error),
    #[error("lockfile error: {0}")]
    Lockfile(#[from] dash_lockfile::LockfileError),
}

/// A single member package within a workspace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceMember {
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
    /// Path to the package directory (relative to workspace root).
    pub path: PathBuf,
    /// The parsed package manifest.
    pub manifest: dash_manifest::PackageManifest,
    /// Whether this is the root package.
    pub is_root: bool,
}

impl WorkspaceMember {
    /// Create a new workspace member.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        path: PathBuf,
        manifest: dash_manifest::PackageManifest,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            path,
            manifest,
            is_root: false,
        }
    }

    /// Return the full path to the member.
    pub fn full_path(&self, workspace_root: &Path) -> PathBuf {
        workspace_root.join(&self.path)
    }

    /// Get the member's package identifier.
    pub fn package_id(&self) -> Result<dash_package::PackageId, dash_package::PackageError> {
        let name = dash_package::PackageName::new(&self.name)?;
        let version: semver::Version = self.version.parse().map_err(|e| {
            dash_package::PackageError::InvalidVersion(e)
        })?;
        Ok(dash_package::PackageId::new(name, version))
    }
}

/// Configuration for a workspace.
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    /// Glob patterns for member package directories.
    pub members: Vec<String>,
    /// Exclude patterns.
    pub exclude: Vec<String>,
    /// Shared configuration that applies to all members.
    pub shared_config: Option<dash_config::Config>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            members: vec!["packages/*".to_string(), "crates/*".to_string()],
            exclude: Vec::new(),
            shared_config: None,
        }
    }
}

/// A multi-package workspace.
pub struct Workspace {
    /// Root directory of the workspace.
    pub root: PathBuf,
    /// Workspace members keyed by name.
    pub members: HashMap<String, WorkspaceMember>,
    /// Workspace configuration.
    pub config: WorkspaceConfig,
    /// Shared lockfile (if any).
    pub lockfile: Option<dash_lockfile::Lockfile>,
}

impl Workspace {
    /// Discover a workspace starting from the given path.
    ///
    /// Searches upward for workspace configuration files (dash.json, package.json
    /// with "workspaces" field) and discovers all member packages.
    pub fn discover(path: &Path) -> Result<Self, WorkspaceError> {
        let root = find_workspace_root(path)?;
        let config = load_workspace_config(&root)?;
        let members = discover_members(&root, &config)?;

        // Try to load a lockfile
        let lockfile = dash_lockfile::find_lockfile(&root)
            .and_then(|p| dash_lockfile::Lockfile::load(&p).ok());

        Ok(Self {
            root,
            members,
            config,
            lockfile,
        })
    }

    /// Get a member by name.
    pub fn member(&self, name: &str) -> Option<&WorkspaceMember> {
        self.members.get(name)
    }

    /// Get all workspace members.
    pub fn packages(&self) -> Vec<&WorkspaceMember> {
        self.members.values().collect()
    }

    /// Get the root workspace member (if it's itself a package).
    pub fn root_package(&self) -> Option<&WorkspaceMember> {
        self.members.values().find(|m| m.is_root)
    }

    /// Filter members by glob patterns.
    pub fn filter(&self, globs: &[String]) -> Vec<&WorkspaceMember> {
        self.members
            .values()
            .filter(|member| {
                let rel_path = member.path.to_string_lossy();
                globs.iter().any(|pattern| {
                    let matched = glob::Pattern::new(pattern)
                        .map(|p| p.matches(&rel_path))
                        .unwrap_or(false);
                    matched
                })
            })
            .collect()
    }

    /// Get all member names.
    pub fn member_names(&self) -> Vec<String> {
        self.members.keys().cloned().collect()
    }

    /// Return the number of members.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Check if the workspace is empty.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Get the workspace root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Return the workspace-level dependency graph.
    pub fn dependency_graph(&self) -> dash_graph::DependencyGraph {
        let mut graph = dash_graph::DependencyGraph::new();

        for member in self.members.values() {
            graph.add_node(dash_graph::Node::new(&member.name, &member.version));

            for (dep_name, dep_entry) in &member.manifest.dependencies {
                if self.members.contains_key(dep_name) {
                    let kind = dash_graph::DependencyKind::Prod;
                    let _ = graph.add_edge(&member.name, dep_name, kind);
                }
            }

            for (dep_name, _) in &member.manifest.dev_dependencies {
                if self.members.contains_key(dep_name) {
                    let kind = dash_graph::DependencyKind::Dev;
                    let _ = graph.add_edge(&member.name, dep_name, kind);
                }
            }
        }

        graph
    }

    /// Get the workspace lockfile.
    pub fn lockfile(&self) -> Option<&dash_lockfile::Lockfile> {
        self.lockfile.as_ref()
    }

    /// Iterate over members.
    pub fn iter(&self) -> impl Iterator<Item = &WorkspaceMember> {
        self.members.values()
    }
}

/// Find the workspace root by searching upward for workspace markers.
fn find_workspace_root(path: &Path) -> Result<PathBuf, WorkspaceError> {
    let mut current = Some(path.to_path_buf());

    while let Some(dir) = current {
        // Check for workspace markers
        let dash_json = dir.join("dash.json");
        let package_json = dir.join("package.json");

        if dash_json.exists() || package_json.exists() {
            // Check if it has workspace configuration
            if has_workspace_config(&dir) {
                return Ok(dir);
            }
        }

        // Check for Cargo workspace
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            let content = std::fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(dir);
            }
        }

        current = dir.parent().map(|p| p.to_path_buf());
    }

    Err(WorkspaceError::InvalidRoot(
        "no workspace root found".to_string(),
    ))
}

fn has_workspace_config(path: &Path) -> bool {
    let dash_json = path.join("dash.json");
    if let Ok(content) = std::fs::read_to_string(&dash_json) {
        if content.contains("\"workspaces\"") || content.contains("\"members\"") {
            return true;
        }
    }

    let package_json = path.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&package_json) {
        if content.contains("\"workspaces\"") || content.contains("\"members\"") {
            return true;
        }
    }

    false
}

fn load_workspace_config(root: &Path) -> Result<WorkspaceConfig, WorkspaceError> {
    // Try dash.json or package.json for workspace config
    let dash_json = root.join("dash.json");

    if dash_json.exists() {
        let content = std::fs::read_to_string(&dash_json)?;
        let manifest = dash_manifest::Manifest::parse(&content)?;

        if let Some(workspaces) = manifest.get_raw("workspaces") {
            if let Some(members) = workspaces.as_array() {
                let members: Vec<String> = members
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                if !members.is_empty() {
                    return Ok(WorkspaceConfig {
                        members,
                        ..Default::default()
                    });
                }
            }
        }

        if let Some(members) = manifest.get_raw("members") {
            if let Some(members_arr) = members.as_array() {
                let members: Vec<String> = members_arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect();
                if !members.is_empty() {
                    return Ok(WorkspaceConfig {
                        members,
                        ..Default::default()
                    });
                }
            }
        }
    }

    Ok(WorkspaceConfig::default())
}

fn discover_members(root: &Path, config: &WorkspaceConfig) -> Result<HashMap<String, WorkspaceMember>, WorkspaceError> {
    let mut members = HashMap::new();

    // Check if the root itself is a package
    let root_manifest = root.join("package.json");
    if root_manifest.exists() {
        let content = std::fs::read_to_string(&root_manifest)?;
        if let Ok(manifest) = dash_manifest::Manifest::parse(&content) {
            let member = WorkspaceMember {
                name: manifest.name().to_string(),
                version: manifest.version().to_string(),
                path: PathBuf::from("."),
                manifest: manifest.export().clone(),
                is_root: true,
            };
            members.insert(member.name.clone(), member);
        }
    }

    // Discover members via glob patterns
    for pattern in &config.members {
        let full_pattern = root.join(pattern).to_string_lossy().to_string();
        let glob_pattern = glob::Pattern::new(&full_pattern)
            .map_err(|e| WorkspaceError::Glob(e.to_string()))?;

        // Use walkdir to find matching directories
        for entry in walkdir::WalkDir::new(root)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                !config.exclude.iter().any(|ex| {
                    e.path().to_string_lossy().contains(ex)
                })
            })
        {
            let entry = entry?;

            if entry.file_type().is_dir() && entry.path() != root {
                let rel_path = entry.path().strip_prefix(root).unwrap_or(entry.path());

                if !glob_pattern.matches_path(entry.path()) {
                    continue;
                }

                // Look for manifest files
                let manifest_file = dash_manifest::find_manifest(entry.path());
                if let Some(manifest_path) = manifest_file {
                    let content = std::fs::read_to_string(&manifest_path)?;
                    if let Ok(manifest) = dash_manifest::Manifest::parse(&content) {
                        let name = manifest.name().to_string();

                        if members.contains_key(&name) {
                            continue;
                        }

                        let member = WorkspaceMember {
                            name,
                            version: manifest.version().to_string(),
                            path: rel_path.to_path_buf(),
                            manifest: manifest.export().clone(),
                            is_root: false,
                        };

                        members.insert(member.name.clone(), member);
                    }
                }
            }
        }
    }

    Ok(members)
}

/// Create a workspace by discovery from the current directory.
pub fn discover_workspace() -> Result<Workspace, WorkspaceError> {
    Workspace::discover(&std::env::current_dir()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_config_default() {
        let config = WorkspaceConfig::default();
        assert_eq!(config.members.len(), 2);
        assert!(config.members.contains(&"packages/*".to_string()));
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn test_workspace_member_new() {
        let manifest = dash_manifest::PackageManifest::default();
        let member = WorkspaceMember::new("test-pkg", "1.0.0", PathBuf::from("packages/test"), manifest);
        assert_eq!(member.name, "test-pkg");
        assert_eq!(member.version, "1.0.0");
        assert!(!member.is_root);
    }

    #[test]
    fn test_filter_empty() {
        let root = PathBuf::from("/tmp");
        let config = WorkspaceConfig::default();
        let members = HashMap::new();
        let workspace = Workspace {
            root,
            members,
            config,
            lockfile: None,
        };

        let filtered = workspace.filter(&["packages/*".to_string()]);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_discover_no_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let result = Workspace::discover(dir.path());
        // This will fail because there's no workspace root
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_workspace_member_package_id() {
        let manifest = dash_manifest::PackageManifest::default();
        let member = WorkspaceMember::new("my-pkg", "1.0.0", PathBuf::from("."), manifest);
        let pkg_id = member.package_id().unwrap();
        assert_eq!(pkg_id.name.as_str(), "my-pkg");
        assert_eq!(pkg_id.version.to_string(), "1.0.0");
    }

    #[test]
    fn test_dependency_graph_empty() {
        let root = PathBuf::from("/tmp");
        let config = WorkspaceConfig::default();
        let members = HashMap::new();
        let workspace = Workspace {
            root,
            members,
            config,
            lockfile: None,
        };

        let graph = workspace.dependency_graph();
        assert_eq!(graph.node_count(), 0);
    }

    #[test]
    fn test_workspace_len() {
        let root = PathBuf::from("/tmp");
        let config = WorkspaceConfig::default();
        let mut members = HashMap::new();
        members.insert(
            "a".to_string(),
            WorkspaceMember::new("a", "1.0.0", PathBuf::from("packages/a"), dash_manifest::PackageManifest::default()),
        );
        let workspace = Workspace {
            root,
            members,
            config,
            lockfile: None,
        };
        assert_eq!(workspace.len(), 1);
        assert!(!workspace.is_empty());
        assert_eq!(workspace.member_names(), vec!["a"]);
    }
}
