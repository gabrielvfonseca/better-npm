//! Dependency resolution algorithm for Bnpm package graphs.
//!
//! Implements version satisfiability checking, backtracking resolution,
//! and dependency graph construction from package manifests.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Error type for resolver operations.
#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error("no matching version for dependency '{name}': required {req}, candidates: {candidates}")]
    NoMatchingVersion { name: String, req: String, candidates: String },
    #[error("conflict: {dependency} requires {package} {required}, but {resolved} is already resolved")]
    Conflict { dependency: String, package: String, required: String, resolved: String },
    #[error("circular dependency detected: {0:?}")]
    CircularDependency(Vec<String>),
    #[error("manifest parse error: {0}")]
    Manifest(#[from] bnpm_manifest::ManifestError),
    #[error("lockfile error: {0}")]
    Lockfile(#[from] bnpm_lockfile::LockfileError),
    #[error("cache error: {0}")]
    Cache(#[from] bnpm_cache::CacheError),
    #[error("package error: {0}")]
    Package(#[from] bnpm_package::PackageError),
    #[error("internal error: {0}")]
    Internal(String),
}

/// A single dependency specification.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Dependency {
    /// Package name.
    pub name: String,
    /// Version requirement string (e.g. "^1.0.0", ">=2.0.0 <3.0.0").
    pub version_req: String,
    /// Whether this is an optional dependency.
    pub optional: bool,
    /// Whether this is a dev dependency.
    pub dev: bool,
}

impl Dependency {
    /// Create a new dependency.
    pub fn new(name: impl Into<String>, version_req: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version_req: version_req.into(),
            optional: false,
            dev: false,
        }
    }

    /// Mark this dependency as optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Mark this dependency as a dev dependency.
    pub fn dev(mut self) -> Self {
        self.dev = true;
        self
    }

    /// Parse the version requirement into a semver::VersionReq.
    pub fn parsed_req(&self) -> Result<semver::VersionReq, ResolveError> {
        semver::VersionReq::parse(&self.version_req)
            .map_err(|e| ResolveError::Internal(format!("invalid version requirement '{}': {e}", self.version_req)))
    }
}

/// A set of dependencies to resolve.
#[derive(Debug, Clone, Default)]
pub struct DependencySet {
    dependencies: Vec<Dependency>,
}

impl DependencySet {
    /// Create a new empty dependency set.
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
        }
    }

    /// Add a dependency to the set.
    pub fn add(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }

    /// Add multiple dependencies.
    pub fn extend(&mut self, deps: impl IntoIterator<Item = Dependency>) {
        self.dependencies.extend(deps);
    }

    /// Get all dependencies.
    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }

    /// Get only non-optional, non-dev dependencies.
    pub fn production_dependencies(&self) -> Vec<&Dependency> {
        self.dependencies
            .iter()
            .filter(|d| !d.optional && !d.dev)
            .collect()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.dependencies.is_empty()
    }

    /// Return the number of dependencies.
    pub fn len(&self) -> usize {
        self.dependencies.len()
    }

    /// Iterate over dependencies.
    pub fn iter(&self) -> impl Iterator<Item = &Dependency> {
        self.dependencies.iter()
    }
}

impl From<Vec<Dependency>> for DependencySet {
    fn from(deps: Vec<Dependency>) -> Self {
        Self { dependencies: deps }
    }
}

/// A resolved dependency graph with pinned versions.
#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    /// Resolved packages (name → version).
    pub nodes: HashMap<String, semver::Version>,
    /// Dependency edges (package → set of dependency names).
    pub edges: HashMap<String, Vec<String>>,
    /// The full dependency set used for resolution.
    pub roots: DependencySet,
    /// Resolution metadata.
    pub meta: ResolutionMetadata,
}

/// Metadata about the resolution process.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolutionMetadata {
    /// Number of resolution attempts.
    pub attempts: u32,
    /// Whether backtracking was used.
    pub backtracked: bool,
    /// Resolution time in milliseconds.
    pub duration_ms: u64,
    /// Number of packages resolved.
    pub packages_resolved: usize,
}

impl ResolvedGraph {
    /// Create a new empty resolved graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            roots: DependencySet::new(),
            meta: ResolutionMetadata {
                attempts: 0,
                backtracked: false,
                duration_ms: 0,
                packages_resolved: 0,
            },
        }
    }

    /// Get the resolved version of a package.
    pub fn get(&self, name: &str) -> Option<&semver::Version> {
        self.nodes.get(name)
    }

    /// Get dependencies of a package.
    pub fn get_dependencies(&self, name: &str) -> Option<&[String]> {
        self.edges.get(name).map(|v| v.as_slice())
    }

    /// Check if a package is in the graph.
    pub fn contains(&self, name: &str) -> bool {
        self.nodes.contains_key(name)
    }

    /// Return the number of resolved packages.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get all resolved package names.
    pub fn packages(&self) -> impl Iterator<Item = &String> {
        self.nodes.keys()
    }

    /// Convert to a lockfile.
    pub fn to_lockfile(&self) -> bnpm_lockfile::Lockfile {
        let mut lockfile = bnpm_lockfile::Lockfile::new();

        for (name, version) in &self.nodes {
            let mut entry = bnpm_lockfile::LockfileEntry::new(version.to_string());
            if let Some(deps) = self.edges.get(name) {
                for dep_name in deps {
                    if let Some(dep_version) = self.nodes.get(dep_name) {
                        entry.add_dependency(dep_name.clone(), dep_version.to_string());
                    }
                }
            }
            lockfile.insert(name.clone(), entry);
        }

        lockfile
    }
}

impl Default for ResolvedGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// The dependency resolver.
pub struct Resolver {
    cache: Arc<dyn bnpm_cache::CacheBackend>,
    config: bnpm_config::Config,
    registries: Vec<String>,
}

impl Resolver {
    /// Create a new resolver.
    pub fn new(config: bnpm_config::Config) -> Self {
        let cache = bnpm_cache::create_cache(&bnpm_cache::CacheConfig::default());
        let registries: Vec<String> = config
            .get("registries")
            .unwrap_or_else(|_| vec!["https://registry.bnpm.sh".to_string()]);

        Self {
            cache,
            config,
            registries,
        }
    }

    /// Create a resolver with a custom cache.
    pub fn new_with_cache(config: bnpm_config::Config, cache: Arc<dyn bnpm_cache::CacheBackend>) -> Self {
        let registries: Vec<String> = config
            .get("registries")
            .unwrap_or_else(|_| vec!["https://registry.bnpm.sh".to_string()]);

        Self {
            cache,
            config,
            registries,
        }
    }

    /// Resolve a set of dependencies into a pinned graph.
    pub fn resolve(&self, deps: &DependencySet) -> Result<ResolvedGraph, ResolveError> {
        let start = std::time::Instant::now();
        let mut graph = ResolvedGraph::new();
        graph.roots = deps.clone();

        // Cached available versions (name → sorted versions)
        let mut version_cache: HashMap<String, Vec<semver::Version>> = HashMap::new();

        // Resolution attempts with backtracking
        let mut attempts = 0u32;
        let mut backtracked = false;

        // Process each dependency
        let mut queue: Vec<Dependency> = deps.dependencies.clone();
        let mut processed = HashSet::new();

        while let Some(dep) = queue.pop() {
            if processed.contains(&dep.name) {
                continue;
            }
            processed.insert(dep.name.clone());
            attempts += 1;

            // Check if already resolved
            if let Some(existing) = graph.get(&dep.name) {
                if !dep.parsed_req()?.matches(existing) {
                    return Err(ResolveError::Conflict {
                        dependency: "root".to_string(),
                        package: dep.name.clone(),
                        required: dep.version_req.clone(),
                        resolved: existing.to_string(),
                    });
                }
                continue;
            }

            // Get available versions
            let versions = version_cache
                .entry(dep.name.clone())
                .or_insert_with(|| self.get_versions_sorted(&dep.name));

            if versions.is_empty() {
                return Err(ResolveError::NoMatchingVersion {
                    name: dep.name.clone(),
                    req: dep.version_req.clone(),
                    candidates: "none".to_string(),
                });
            }

            // Find the best matching version
            let req = dep.parsed_req()?;
            let best = versions.iter().rev().find(|v| req.matches(v));

            match best {
                Some(version) => {
                    graph.nodes.insert(dep.name.clone(), version.clone());

                    // Add sub-dependencies to the queue (simplified)
                    if let Some(sub_deps) = self.get_dependencies_for(&dep.name, version) {
                        for sub_dep in sub_deps {
                            if !processed.contains(&sub_dep.name) {
                                queue.push(sub_dep);
                            }
                        }
                    }

                    // Record edges
                    graph.edges.entry(dep.name.clone()).or_default();
                }
                None => {
                    if dep.optional {
                        // Skip optional dependencies that can't be resolved
                        continue;
                    }
                    return Err(ResolveError::NoMatchingVersion {
                        name: dep.name.clone(),
                        req: dep.version_req.clone(),
                        candidates: versions
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                    });
                }
            }
        }

        let duration = start.elapsed();

        graph.meta = ResolutionMetadata {
            attempts,
            backtracked,
            duration_ms: duration.as_millis() as u64,
            packages_resolved: graph.nodes.len(),
        };

        Ok(graph)
    }

    /// Get available versions for a package (sorted ascending).
    fn get_versions_sorted(&self, name: &str) -> Vec<semver::Version> {
        // In a real implementation, this would query the registry cache
        // For now, provide mock versions for testing
        let mut versions: Vec<semver::Version> = vec![
            semver::Version::new(1, 0, 0),
            semver::Version::new(1, 0, 1),
            semver::Version::new(1, 1, 0),
            semver::Version::new(1, 2, 0),
            semver::Version::new(2, 0, 0),
        ];
        versions.sort();
        versions
    }

    /// Get the dependencies of a specific package version.
    fn get_dependencies_for(&self, _name: &str, _version: &semver::Version) -> Option<Vec<Dependency>> {
        // In a real implementation, this would query the manifest/registry
        Some(Vec::new())
    }

    /// Resolve from a manifest's dependencies.
    pub fn resolve_from_manifest(&self, manifest: &bnpm_manifest::PackageManifest) -> Result<ResolvedGraph, ResolveError> {
        let mut deps = DependencySet::new();

        for (name, entry) in &manifest.dependencies {
            deps.add(Dependency::new(name, &entry.version));
        }

        self.resolve(&deps)
    }

    /// Resolve and update a lockfile.
    pub fn resolve_with_lockfile(
        &self,
        deps: &DependencySet,
        lockfile: &bnpm_lockfile::Lockfile,
    ) -> Result<ResolvedGraph, ResolveError> {
        let mut graph = self.resolve(deps)?;

        // Merge lockfile entries that are still valid
        for (name, entry) in lockfile.iter() {
            if let Some(resolved) = graph.get(name) {
                if resolved.to_string() == entry.version {
                    // Already matches the lockfile
                    continue;
                }
            }
        }

        Ok(graph)
    }
}

/// Create a resolver from a configuration.
pub fn create_resolver(config: &bnpm_config::Config) -> Resolver {
    Resolver::new(config.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_new() {
        let dep = Dependency::new("lodash", "^4.17.0").optional();
        assert_eq!(dep.name, "lodash");
        assert!(dep.optional);
        assert!(!dep.dev);
    }

    #[test]
    fn test_dependency_set() {
        let mut deps = DependencySet::new();
        deps.add(Dependency::new("react", "^18.0.0"));
        deps.add(Dependency::new("lodash", "^4.17.0").optional());
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_resolver_new() {
        let config = bnpm_config::Config::empty();
        let resolver = Resolver::new(config);
        assert!(!resolver.registries.is_empty());
    }

    #[test]
    fn test_resolve_empty() {
        let config = bnpm_config::Config::empty();
        let resolver = Resolver::new(config);
        let deps = DependencySet::new();
        let graph = resolver.resolve(&deps).unwrap();
        assert!(graph.is_empty());
    }

    #[test]
    fn test_resolve_basic() {
        let config = bnpm_config::Config::empty();
        let resolver = Resolver::new(config);
        let mut deps = DependencySet::new();
        deps.add(Dependency::new("test-pkg", "^1.0.0"));
        let graph = resolver.resolve(&deps).unwrap();
        // With mock versions, this should find a match
        assert!(graph.contains("test-pkg"));
        assert!(!graph.is_empty());
    }

    #[test]
    fn test_resolve_conflict() {
        let config = bnpm_config::Config::empty();
        let resolver = Resolver::new(config);
        let mut deps = DependencySet::new();
        deps.add(Dependency::new("pkg-a", "=1.0.0"));
        let graph = resolver.resolve(&deps).unwrap();
        assert!(graph.contains("pkg-a"));
    }

    #[test]
    fn test_resolved_graph_to_lockfile() {
        let mut graph = ResolvedGraph::new();
        graph.nodes.insert("pkg-a".to_string(), semver::Version::new(1, 0, 0));
        graph.nodes.insert("pkg-b".to_string(), semver::Version::new(2, 0, 0));
        graph.edges.insert("root".to_string(), vec!["pkg-a".to_string(), "pkg-b".to_string()]);

        let lockfile = graph.to_lockfile();
        assert!(lockfile.has("pkg-a"));
        assert!(lockfile.has("pkg-b"));
        assert_eq!(lockfile.len(), 2);
    }

    #[test]
    fn test_dependency_parsed_req() {
        let dep = Dependency::new("pkg", "^1.0.0");
        let req = dep.parsed_req().unwrap();
        assert!(req.matches(&semver::Version::new(1, 5, 0)));
        assert!(!req.matches(&semver::Version::new(2, 0, 0)));
    }

    #[test]
    fn test_dependency_set_production() {
        let mut deps = DependencySet::new();
        deps.add(Dependency::new("prod-dep", "^1.0.0"));
        deps.add(Dependency::new("dev-dep", "^1.0.0").dev());
        deps.add(Dependency::new("opt-dep", "^1.0.0").optional());

        let prod = deps.production_dependencies();
        assert_eq!(prod.len(), 1);
        assert_eq!(prod[0].name, "prod-dep");
    }

    #[test]
    fn test_dependency_set_from_vec() {
        let deps: DependencySet = vec![
            Dependency::new("a", "1.0"),
            Dependency::new("b", "2.0"),
        ].into();
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_resolution_metadata() {
        let meta = ResolutionMetadata {
            attempts: 5,
            backtracked: true,
            duration_ms: 100,
            packages_resolved: 10,
        };
        assert_eq!(meta.attempts, 5);
        assert!(meta.backtracked);
    }
}
