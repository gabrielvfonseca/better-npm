use std::collections::BTreeMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use bnpm_cid::Cid;
use bnpm_graph::DependencyKind;
use bnpm_lockfile::Lockfile;
use bnpm_manifest::{DependencyKind as ManifestDependencyKind, PackageManifest};
use bnpm_package::PackageId;
use semver::Version;

/// Configuration for a package registry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryConfig {
    /// Registry URL.
    pub url: String,
    /// Authentication token (optional).
    pub token: Option<String>,
    /// Priority order (lower = higher priority).
    pub priority: u32,
    /// Request timeout in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Whether to verify SSL certificates.
    #[serde(default = "default_true")]
    pub verify_ssl: bool,
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

/// Options for the resolver.
#[derive(Clone)]
pub struct ResolveOptions {
    /// The root package manifest.
    pub root_manifest: PackageManifest,
    /// Registry configurations in priority order.
    pub registries: Vec<RegistryConfig>,
    /// Cache backend for storing registry indices and manifests.
    pub cache: Arc<dyn bnpm_cache::CacheBackend>,
    /// Whether to run in offline mode (cache only).
    pub offline: bool,
    /// Maximum number of parallel registry requests.
    pub max_parallel: usize,
    /// Include yanked versions in resolution.
    pub include_yanked: bool,
}

#[allow(dead_code)]
fn default_max_parallel() -> usize {
    10
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self {
            root_manifest: PackageManifest::default(),
            registries: vec![],
            cache: bnpm_cache::create_cache(&bnpm_cache::CacheConfig::default()),
            offline: false,
            max_parallel: 10,
            include_yanked: false,
        }
    }
}

/// A resolved package node in the dependency graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolvedNode {
    /// Package identifier (name@version).
    pub package_id: PackageId,
    /// The resolved version.
    pub version: Version,
    /// The resolved manifest for this version.
    pub manifest: PackageManifest,
    /// Content ID if already cached/fetched.
    pub cid: Option<Cid>,
    /// Dependencies of this package.
    pub dependencies: Vec<DependencyEdge>,
    /// Registry URL from which this package was resolved.
    #[serde(default)]
    pub registry: Option<String>,
    /// Provenance record from the transparency log.
    #[serde(default)]
    pub provenance: Option<bnpm_lockfile::Provenance>,
    /// Capabilities declared by this package.
    #[serde(default)]
    pub capabilities: Option<bnpm_lockfile::LockCapabilities>,
}

/// A dependency edge in the resolved graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DependencyEdge {
    /// Dependency package name.
    pub dep_name: String,
    /// Dependency package identifier.
    pub dep_id: PackageId,
    /// Kind of dependency (prod, dev, optional, peer).
    pub kind: DependencyKind,
    /// The package that requires this dependency.
    pub required_by: PackageId,
    /// The version requirement that selected this version.
    pub required_range: String,
}

/// The fully resolved dependency graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolvedGraph {
    /// Root package identifier.
    pub root: PackageId,
    /// All resolved nodes, keyed by package name@version.
    pub nodes: BTreeMap<PackageId, ResolvedNode>,
    /// Dependency edges.
    pub edges: Vec<(PackageId, PackageId)>,
    /// When the resolution was generated.
    pub generated_at: DateTime<Utc>,
    /// Resolver version.
    pub resolver_version: String,
    /// The version requirements that led to each selected version.
    pub version_requirements: BTreeMap<String, VersionReqSource>,
}

/// Source of a version requirement for traceability.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VersionReqSource {
    /// The version requirement string.
    pub requirement: String,
    /// The package that imposed this requirement.
    pub required_by: PackageId,
    /// The kind of dependency.
    pub kind: DependencyKind,
}

/// Registry index entry for a package version.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryIndexEntry {
    /// Package name.
    pub name: String,
    /// Version string.
    pub version: String,
    /// Content ID of the package capsule.
    pub cid: Option<String>,
    /// Whether this version is yanked.
    #[serde(default)]
    pub yanked: bool,
    /// Dependencies for this version.
    pub dependencies: BTreeMap<String, String>,
    /// Dev dependencies for this version.
    pub dev_dependencies: BTreeMap<String, String>,
    /// Optional dependencies for this version.
    pub optional_dependencies: BTreeMap<String, String>,
    /// Peer dependencies for this version.
    pub peer_dependencies: BTreeMap<String, String>,
    /// Distribution tarball URL.
    pub tarball: Option<String>,
    /// Integrity hash.
    pub integrity: Option<String>,
}

/// Registry index for a package (all versions).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryIndex {
    /// Package name.
    pub name: String,
    /// All versions, sorted by version descending.
    pub versions: Vec<RegistryIndexEntry>,
    /// ETag for conditional requests.
    pub etag: Option<String>,
    /// Last modified timestamp.
    pub last_modified: Option<DateTime<Utc>>,
}

/// A version requirement with its source information for MVS.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VersionRequirement {
    /// Package name.
    pub name: String,
    /// Version requirement (semver range).
    pub requirement: semver::VersionReq,
    /// Kind of dependency.
    pub kind: DependencyKind,
    /// Whether this dependency must be a singleton (unified across graph).
    #[serde(default)]
    pub singleton: bool,
    /// Package that requires this dependency.
    pub required_by: PackageId,
}

impl ResolvedGraph {
    /// Create a new empty resolved graph.
    pub fn new(root: PackageId) -> Self {
        Self {
            root,
            nodes: BTreeMap::new(),
            edges: Vec::new(),
            generated_at: Utc::now(),
            resolver_version: env!("CARGO_PKG_VERSION").to_string(),
            version_requirements: BTreeMap::new(),
        }
    }

    /// Get a node by package ID.
    pub fn get_node(&self, id: &PackageId) -> Option<&ResolvedNode> {
        self.nodes.get(id)
    }

    /// Get all production dependency edges.
    pub fn production_edges(&self) -> Vec<&(PackageId, PackageId)> {
        self.edges
            .iter()
            .filter(|(from, to)| {
                self.nodes
                    .get(from)
                    .and_then(|n| n.dependencies.iter().find(|d| &d.dep_id == to))
                    .map(|d| d.kind == DependencyKind::Prod)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get all package IDs in the graph.
    pub fn package_ids(&self) -> Vec<PackageId> {
        self.nodes.keys().cloned().collect()
    }

    /// Convert to a bnpm-graph DependencyGraph.
    pub fn to_dependency_graph(&self) -> bnpm_graph::DependencyGraph {
        let mut graph = bnpm_graph::DependencyGraph::new();

        for node in self.nodes.values() {
            graph.add_node(bnpm_graph::Node::new(
                node.package_id.name.as_str(),
                node.package_id.version.to_string(),
            ));
        }

        for (from, to) in &self.edges {
            if let (Some(from_node), Some(to_node)) = (self.nodes.get(from), self.nodes.get(to)) {
                if let Some(edge) = from_node.dependencies.iter().find(|d| &d.dep_id == to) {
                    let _ = graph.add_edge(
                        from_node.package_id.name.as_str(),
                        to_node.package_id.name.as_str(),
                        edge.kind,
                    );
                }
            }
        }

        graph
    }

    pub fn to_lockfile(&self) -> Result<Lockfile, bnpm_lockfile::LockfileError> {
        let mut lockfile = Lockfile::with_root(self.root.to_specifier());

        let mut packages = Vec::new();

        for node in self.nodes.values() {
            let cid = match &node.cid {
                Some(c) => c.to_string(),
                None => {
                    return Err(bnpm_lockfile::LockfileError::InvalidEntry(format!(
                        "missing CID for {}@{}",
                        node.package_id.name, node.version
                    )));
                },
            };

            let resolved_from = self
                .edges
                .iter()
                .find(|(_, to)| to == &node.package_id)
                .and_then(|(from_id, _)| {
                    self.nodes
                        .get(from_id)?
                        .dependencies
                        .iter()
                        .find(|d| d.dep_id == node.package_id)
                        .map(|d| d.required_range.clone())
                })
                .unwrap_or_else(|| node.version.to_string());

            let registry = node.registry.clone().unwrap_or_else(|| "bnpm:default".into());

            let pkg = bnpm_lockfile::LockPackage {
                name: node.package_id.name.to_string(),
                version: node.version.to_string(),
                cid,
                resolved_from,
                registry,
                provenance: node.provenance.clone(),
                capabilities: node.capabilities.clone(),
                build: None,
            };

            packages.push(pkg);
        }

        packages.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));
        lockfile.package = packages;

        for node in self.nodes.values() {
            let key = format!("{}@{}", node.package_id.name, node.version);
            let mut deps: Vec<String> = node
                .dependencies
                .iter()
                .map(|d| format!("{}@{}", d.dep_name, d.dep_id.version))
                .collect();
            deps.sort();
            lockfile.graph.insert(key, deps);
        }

        lockfile.finalize();

        Ok(lockfile)
    }

    /// Return the trace of which edge(s) set the selected version for a package.
    pub fn why(&self, name: &str) -> Vec<WhyEdge> {
        self.nodes
            .values()
            .filter(|n| n.package_id.name.as_str() == name)
            .flat_map(|node| {
                self.version_requirements
                    .iter()
                    .filter(|(n, _)| *n == name)
                    .map(|(_, src)| WhyEdge {
                        required_by: src.required_by.clone(),
                        range: src.requirement.clone(),
                        selected: node.version.clone(),
                        kind: src.kind,
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

/// Convert a Manifest DependencyKind to graph DependencyKind.
pub fn convert_dep_kind(kind: ManifestDependencyKind) -> DependencyKind {
    match kind {
        ManifestDependencyKind::Prod => DependencyKind::Prod,
        ManifestDependencyKind::Dev => DependencyKind::Dev,
        ManifestDependencyKind::Optional => DependencyKind::Optional,
        ManifestDependencyKind::Peer => DependencyKind::Peer,
    }
}

/// A trace edge explaining why a version was selected.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WhyEdge {
    pub required_by: PackageId,
    pub range: String,
    pub selected: Version,
    pub kind: DependencyKind,
}

/// A single requirement in a conflict explanation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConflictRequirement {
    pub requirer: String,
    pub range: String,
}

/// A structured explanation of why a dependency conflict cannot be resolved.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConflictExplanation {
    pub package: String,
    pub requirements: Vec<ConflictRequirement>,
    pub summary: String,
}

impl std::fmt::Display for ConflictExplanation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let reqs: Vec<String> = self
            .requirements
            .iter()
            .map(|r| format!("`{}` needs `{}`", r.requirer, r.range))
            .collect();
        write!(
            f,
            "cannot satisfy `{}`: {}; no single version satisfies all (singleton)",
            self.package,
            reqs.join(", ")
        )
    }
}

/// Error type for resolver operations.
#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    #[error(
        "no matching version for dependency '{name}': required {req}, available versions: {candidates}"
    )]
    NoMatchingVersion { name: String, req: String, candidates: String },
    #[error(
        "conflict: {dependency} requires {package} {required}, but {resolved} is already resolved"
    )]
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
    #[error("registry error: {0}")]
    Registry(#[from] bnpm_registry_client::RegistryError),
    #[error("CID error: {0}")]
    Cid(#[from] bnpm_cid::CidError),
    #[error("graph error: {0}")]
    Graph(#[from] bnpm_graph::GraphError),
    #[error("semver error: {0}")]
    Semver(#[from] semver::Error),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("internal error: {0}")]
    Internal(String),
    #[error(
        "peer dependency conflict: {package} requires peer {peer} {required}, but {resolved} is resolved"
    )]
    PeerConflict { package: String, peer: String, required: String, resolved: String },
    #[error("yanked version selected: {name}@{version} (use --include-yanked to allow)")]
    YankedVersion { name: String, version: String },
    #[error("missing manifest for {name}@{version}")]
    MissingManifest { name: String, version: String },
    #[error("offline mode: {name}@{version} not in cache")]
    OfflineMissing { name: String, version: String },
    #[error("version requirement parse error: {0}")]
    VersionReqParse(String),
    #[error(
        "singleton conflict: dependency '{name}' is marked singleton but requires two major versions. '{requirer_a}' requires {req_a}, '{requirer_b}' requires {req_b}"
    )]
    SingletonConflict {
        name: String,
        requirer_a: String,
        req_a: String,
        requirer_b: String,
        req_b: String,
    },
    #[error("{0}")]
    ConflictExplanation(ConflictExplanation),
}

impl ResolveOptions {
    /// Create new resolve options from a manifest.
    pub fn new(root_manifest: PackageManifest) -> Self {
        Self { root_manifest, ..Default::default() }
    }

    /// Add a registry configuration.
    pub fn with_registry(mut self, registry: RegistryConfig) -> Self {
        self.registries.push(registry);
        self
    }

    /// Set the cache backend.
    pub fn with_cache(mut self, cache: Arc<dyn bnpm_cache::CacheBackend>) -> Self {
        self.cache = cache;
        self
    }

    /// Set offline mode.
    pub fn offline(mut self, offline: bool) -> Self {
        self.offline = offline;
        self
    }

    /// Set maximum parallel requests.
    pub fn max_parallel(mut self, max: usize) -> Self {
        self.max_parallel = max;
        self
    }

    /// Include yanked versions.
    pub fn include_yanked(mut self, include: bool) -> Self {
        self.include_yanked = include;
        self
    }
}

/// Result of a resolution operation.
#[derive(Debug, Clone)]
pub struct ResolutionResult {
    /// The resolved graph.
    pub graph: ResolvedGraph,
    /// Resolution metadata.
    pub metadata: ResolutionMetadata,
}

/// Metadata about the resolution process.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolutionMetadata {
    /// Number of resolution attempts.
    pub attempts: u32,
    /// Whether backtracking occurred.
    pub backtracked: bool,
    /// Resolution duration in milliseconds.
    pub duration_ms: u64,
    /// Number of packages resolved.
    pub packages_resolved: usize,
    /// Number of registry requests made.
    pub registry_requests: usize,
    /// Number of cache hits.
    pub cache_hits: usize,
}
