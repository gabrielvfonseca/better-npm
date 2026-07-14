use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dash_resolver::ResolvedGraph;
use dash_store::CasStore;

use crate::InstallError;

const VIRTUAL_STORE_DIR: &str = ".dash";

/// Materialize a resolved graph into a strict, non-hoisted `node_modules` tree
/// in the pnpm style. Each package's files are materialized from the CAS store
/// into a virtual store dir, and per-package symlink trees are created so that
/// a package can only import its declared (direct) dependencies.
///
/// # Layout
///
/// ```text
/// node_modules/
///   .dash/
///     <name>@<version>/
///       node_modules/
///         <name>/            ← actual files (materialized from CAS)
///         <dep>/ -> ../../<dep>@<ver>/node_modules/<dep>/   ← symlink
///   <root-dep>/ -> .dash/<dep>@<ver>/node_modules/<dep>/    ← symlink
/// ```
///
/// The root gets only its own direct dependencies at the top level.
/// There is no hoisting — phantom dependencies are structurally impossible.
pub async fn materialize_node_modules(
    graph: &ResolvedGraph,
    store: &CasStore,
    project_root: &Path,
) -> Result<(), InstallError> {
    let node_modules = project_root.join("node_modules");
    let virtual_store = node_modules.join(VIRTUAL_STORE_DIR);

    tokio::fs::create_dir_all(&virtual_store).await?;

    let direct_deps = build_direct_dep_map(graph);

    for (pkg_id, node) in &graph.nodes {
        let cid = node
            .cid
            .as_ref()
            .ok_or_else(|| InstallError::Validation(format!("missing CID for {pkg_id}")))?;

        let pkg_key = pkg_id.to_string();
        let pkg_node_modules = virtual_store.join(&pkg_key).join("node_modules");
        let pkg_content_dir = pkg_node_modules.join(pkg_id.name.as_str());

        if pkg_content_dir.exists() {
            let _ = tokio::fs::remove_dir_all(&pkg_content_dir).await;
        }
        store.materialize(cid, &pkg_content_dir).await?;

        if let Some(deps) = direct_deps.get(&pkg_id.to_string()) {
            for dep_id in deps {
                let dep_key = dep_id.to_string();
                let dep_target =
                    virtual_store.join(&dep_key).join("node_modules").join(dep_id.name.as_str());
                let link_path = pkg_node_modules.join(dep_id.name.as_str());

                create_relative_symlink(&dep_target, &link_path)?;
            }
        }
    }

    let root_deps = direct_deps.get(&graph.root.to_string());
    if let Some(deps) = root_deps {
        for dep_id in deps {
            let dep_key = dep_id.to_string();
            let dep_target =
                virtual_store.join(&dep_key).join("node_modules").join(dep_id.name.as_str());
            let link_path = node_modules.join(dep_id.name.as_str());

            create_relative_symlink(&dep_target, &link_path)?;
        }
    }

    Ok(())
}

fn build_direct_dep_map(graph: &ResolvedGraph) -> HashMap<String, Vec<dash_package::PackageId>> {
    let mut map: HashMap<String, Vec<dash_package::PackageId>> = HashMap::new();

    for (from, to) in &graph.edges {
        map.entry(from.to_string()).or_default().push(to.clone());
    }

    map
}

fn create_relative_symlink(target: &Path, link: &Path) -> Result<(), std::io::Error> {
    if link.is_symlink() {
        let _ = std::fs::remove_file(link);
    }

    let link_parent = link.parent().unwrap_or_else(|| Path::new("."));
    let relative = compute_relative_path(link_parent, target);

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&relative, link)?;
    }
    #[cfg(not(unix))]
    {
        std::os::windows::fs::symlink_dir(&relative, link)?;
    }

    Ok(())
}

fn compute_relative_path(from: &Path, to: &Path) -> PathBuf {
    let from = from.canonicalize().unwrap_or_else(|_| from.to_path_buf());
    let to = to.canonicalize().unwrap_or_else(|_| to.to_path_buf());

    let from_components: Vec<_> = from.components().collect();
    let to_components: Vec<_> = to.components().collect();

    let common_len = from_components
        .iter()
        .zip(to_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let up_count = from_components.len() - common_len;
    let mut result = PathBuf::new();
    for _ in 0..up_count {
        result.push("..");
    }
    for component in &to_components[common_len..] {
        result.push(component);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use dash_capsule::Capsule;
    use dash_cid::Cid;
    use dash_graph::DependencyKind;
    use dash_package::{PackageId, PackageName};
    use dash_resolver::{DependencyEdge, ResolvedGraph, ResolvedNode};
    use dash_store::CasStore;
    use semver::Version;
    use tempfile::TempDir;

    type StoreFixture = (CasStore, TempDir, HashMap<String, Cid>);

    type GraphNodeSpec = (PackageId, Option<Cid>, Vec<(String, PackageId)>);

    fn make_id(name: &str, version: &str) -> PackageId {
        PackageId::new(PackageName::new_unchecked(name), Version::parse(version).unwrap())
    }

    fn make_capsule(name: &str) -> Capsule {
        let content = format!("// package {name}\nexport const x = 1;\n");
        Capsule::pack(vec![
            ("package.json".to_string(), format!("{{\"name\":\"{name}\"}}").into_bytes()),
            ("index.js".to_string(), content.into_bytes()),
        ])
        .unwrap()
    }

    async fn setup_store(capsules: &[(PackageId, Capsule)]) -> StoreFixture {
        let dir = TempDir::new().unwrap();
        let store = CasStore::new(dir.path().to_path_buf()).await.unwrap();
        let mut cids = HashMap::new();

        for (pkg_id, capsule) in capsules {
            let cid = store.store_capsule(capsule).await.unwrap();
            cids.insert(pkg_id.to_string(), cid);
        }

        (store, dir, cids)
    }

    fn build_graph(
        root: PackageId,
        nodes: Vec<GraphNodeSpec>,
        edges: Vec<(PackageId, PackageId)>,
    ) -> ResolvedGraph {
        let mut graph = ResolvedGraph::new(root);
        graph.edges = edges;

        for (pkg_id, cid, deps) in nodes {
            let version = pkg_id.version.clone();
            let dep_edges: Vec<DependencyEdge> = deps
                .into_iter()
                .map(|(dep_name, dep_id)| DependencyEdge {
                    dep_name,
                    dep_id,
                    kind: DependencyKind::Prod,
                    required_by: pkg_id.clone(),
                    required_range: "*".to_string(),
                })
                .collect();

            graph.nodes.insert(
                pkg_id.clone(),
                ResolvedNode {
                    package_id: pkg_id,
                    version,
                    manifest: Default::default(),
                    cid,
                    dependencies: dep_edges,
                    registry: None,
                    provenance: None,
                    capabilities: None,
                },
            );
        }

        graph
    }

    /// Graph: root -> a -> b. Root does NOT depend on b.
    /// Verify non-hoisted layout and phantom-dep prevention.
    #[tokio::test]
    async fn test_non_hoisted_layout() {
        let root = make_id("root", "0.0.0");
        let a_id = make_id("a", "1.0.0");
        let b_id = make_id("b", "1.0.0");

        let cap_a = make_capsule("a");
        let cap_b = make_capsule("b");

        let (store, _store_dir, cids) =
            setup_store(&[(a_id.clone(), cap_a), (b_id.clone(), cap_b)]).await;

        let graph = build_graph(
            root.clone(),
            vec![
                (
                    a_id.clone(),
                    Some(cids.get(&a_id.to_string()).unwrap().clone()),
                    vec![("b".to_string(), b_id.clone())],
                ),
                (b_id.clone(), Some(cids.get(&b_id.to_string()).unwrap().clone()), vec![]),
            ],
            vec![(root.clone(), a_id.clone()), (a_id.clone(), b_id.clone())],
        );

        let project = TempDir::new().unwrap();
        materialize_node_modules(&graph, &store, project.path()).await.unwrap();

        let nm = project.path().join("node_modules");

        // Top-level: a exists, b does NOT (phantom-dep prevention)
        assert!(nm.join("a").is_symlink(), "top-level a should be a symlink");
        assert!(!nm.join("b").exists(), "top-level b must NOT exist (phantom dep prevention)");

        // .dash/a@1.0.0/node_modules/a/ has actual files
        let a_virtual = nm.join(".dash/a@1.0.0/node_modules/a");
        assert!(a_virtual.is_dir(), "virtual a dir should exist with actual files");
        assert!(a_virtual.join("index.js").exists(), "a should have index.js");

        // .dash/a@1.0.0/node_modules/b/ -> symlink to b's virtual dir
        let a_b_link = nm.join(".dash/a@1.0.0/node_modules/b");
        assert!(a_b_link.is_symlink(), "a's node_modules/b should be a symlink to b");

        // .dash/b@1.0.0/node_modules/b/ has actual files
        let b_virtual = nm.join(".dash/b@1.0.0/node_modules/b");
        assert!(b_virtual.is_dir(), "virtual b dir should exist with actual files");
        assert!(b_virtual.join("index.js").exists(), "b should have index.js");

        // Read b's content through the symlink from a
        let resolved = a_b_link.canonicalize().unwrap();
        assert!(resolved.join("index.js").exists(), "should resolve to b's real files");
    }

    /// Verify phantom-dep prevention: root cannot see b at top level.
    #[tokio::test]
    async fn test_phantom_dep_prevention() {
        let root = make_id("root", "0.0.0");
        let a_id = make_id("dep-a", "1.0.0");
        let b_id = make_id("dep-b", "1.0.0");

        let cap_a = make_capsule("dep-a");
        let cap_b = make_capsule("dep-b");

        let (store, _store_dir, cids) =
            setup_store(&[(a_id.clone(), cap_a), (b_id.clone(), cap_b)]).await;

        let graph = build_graph(
            root.clone(),
            vec![
                (
                    a_id.clone(),
                    Some(cids.get(&a_id.to_string()).unwrap().clone()),
                    vec![("dep-b".to_string(), b_id.clone())],
                ),
                (b_id.clone(), Some(cids.get(&b_id.to_string()).unwrap().clone()), vec![]),
            ],
            vec![(root.clone(), a_id.clone()), (a_id.clone(), b_id.clone())],
        );

        let project = TempDir::new().unwrap();
        materialize_node_modules(&graph, &store, project.path()).await.unwrap();

        let nm = project.path().join("node_modules");

        // root can see dep-a
        assert!(nm.join("dep-a").is_symlink());

        // root CANNOT see dep-b (it's a phantom)
        assert!(!nm.join("dep-b").exists(), "phantom dep-b must NOT be visible at top level");

        // dep-a CAN see dep-b
        let a_b_link = nm.join(".dash/dep-a@1.0.0/node_modules/dep-b");
        assert!(a_b_link.is_symlink(), "dep-a should see dep-b");
        assert!(a_b_link.canonicalize().unwrap().join("index.js").exists());

        // dep-b has only its own content, no deps
        let b_nm = nm.join(".dash/dep-b@1.0.0/node_modules");
        let b_entries: Vec<_> = std::fs::read_dir(&b_nm)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        assert!(b_entries.contains(&"dep-b".to_string()));
        assert!(!b_entries.iter().any(|s| s != "dep-b"));
    }

    /// Running materialize twice yields the same structure (idempotent).
    #[tokio::test]
    async fn test_idempotent_materialize() {
        let root = make_id("root", "0.0.0");
        let a_id = make_id("p", "1.0.0");
        let cap = make_capsule("p");
        let (store, _store_dir, cids) = setup_store(&[(a_id.clone(), cap)]).await;

        let graph = build_graph(
            root.clone(),
            vec![(a_id.clone(), Some(cids.get(&a_id.to_string()).unwrap().clone()), vec![])],
            vec![(root.clone(), a_id.clone())],
        );

        let project = TempDir::new().unwrap();

        // First run
        materialize_node_modules(&graph, &store, project.path()).await.unwrap();

        let idx_after_first =
            std::fs::read_to_string(project.path().join("node_modules/p/index.js")).unwrap();

        // Second run
        materialize_node_modules(&graph, &store, project.path()).await.unwrap();

        let idx_after_second =
            std::fs::read_to_string(project.path().join("node_modules/p/index.js")).unwrap();

        assert_eq!(idx_after_first, idx_after_second);
        assert!(project.path().join("node_modules/p").is_symlink());
    }

    /// Two packages depending on the same CID materialize the underlying
    /// files once in the virtual store.
    #[tokio::test]
    async fn test_shared_dep_dedup() {
        let root = make_id("root", "0.0.0");
        let a_id = make_id("a", "1.0.0");
        let b_id = make_id("b", "1.0.0");
        let shared_id = make_id("shared-lib", "2.0.0");

        let cap_a = make_capsule("a");
        let cap_b = make_capsule("b");
        let cap_shared = make_capsule("shared-lib");

        let (store, _store_dir, cids) = setup_store(&[
            (a_id.clone(), cap_a),
            (b_id.clone(), cap_b),
            (shared_id.clone(), cap_shared),
        ])
        .await;

        let graph = build_graph(
            root.clone(),
            vec![
                (
                    a_id.clone(),
                    Some(cids.get(&a_id.to_string()).unwrap().clone()),
                    vec![("shared-lib".to_string(), shared_id.clone())],
                ),
                (
                    b_id.clone(),
                    Some(cids.get(&b_id.to_string()).unwrap().clone()),
                    vec![("shared-lib".to_string(), shared_id.clone())],
                ),
                (
                    shared_id.clone(),
                    Some(cids.get(&shared_id.to_string()).unwrap().clone()),
                    vec![],
                ),
            ],
            vec![
                (root.clone(), a_id.clone()),
                (root.clone(), b_id.clone()),
                (a_id.clone(), shared_id.clone()),
                (b_id.clone(), shared_id.clone()),
            ],
        );

        let project = TempDir::new().unwrap();
        materialize_node_modules(&graph, &store, project.path()).await.unwrap();

        let nm = project.path().join("node_modules");

        // Both a and b should see shared-lib
        assert!(nm.join(".dash/a@1.0.0/node_modules/shared-lib").is_symlink());
        assert!(nm.join(".dash/b@1.0.0/node_modules/shared-lib").is_symlink());

        // shared-lib should exist exactly once in .dash
        let shared_dir = nm.join(".dash/shared-lib@2.0.0/node_modules/shared-lib");
        assert!(shared_dir.is_dir());
        assert!(shared_dir.join("index.js").exists());

        // Verify both symlinks point to the same real dir
        let resolved_a = nm.join(".dash/a@1.0.0/node_modules/shared-lib").canonicalize().unwrap();
        let resolved_b = nm.join(".dash/b@1.0.0/node_modules/shared-lib").canonicalize().unwrap();
        assert_eq!(resolved_a, resolved_b, "shared dep should point to same underlying dir");
    }

    /// Root has no dependencies: clean empty node_modules (just .dash dir, no top-level entries).
    #[tokio::test]
    async fn test_empty_graph() {
        let root = make_id("root", "0.0.0");
        let graph = build_graph(root.clone(), vec![], vec![]);

        let (store, _store_dir, _cids) = setup_store(&[]).await;

        let project = TempDir::new().unwrap();
        materialize_node_modules(&graph, &store, project.path()).await.unwrap();

        let nm = project.path().join("node_modules");
        assert!(nm.join(".dash").is_dir());

        let top_entries: Vec<_> = std::fs::read_dir(&nm)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .filter(|n| n != ".dash")
            .collect();
        assert!(top_entries.is_empty(), "no top-level symlinks for empty graph");
    }

    /// Missing CID causes a validation error.
    #[tokio::test]
    async fn test_missing_cid_error() {
        let root = make_id("root", "0.0.0");
        let a_id = make_id("a", "1.0.0");

        let graph = build_graph(
            root.clone(),
            vec![(a_id.clone(), None, vec![])],
            vec![(root.clone(), a_id.clone())],
        );

        let (store, _store_dir, _cids) = setup_store(&[]).await;

        let project = TempDir::new().unwrap();
        let result = materialize_node_modules(&graph, &store, project.path()).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("missing CID"), "expected missing CID error, got: {err}");
    }
}
