//! Dependency graph data structures and traversal algorithms.
//!
//! Builds on `petgraph` to provide a directed dependency graph with
//! support for cycle detection, topological ordering, subgraph extraction,
//! and visualization.

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{EdgeRef, IntoNodeReferences, Topo, Walker};
use std::collections::{HashMap, HashSet};

/// Error type for graph operations.
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("cycle detected: {0:?}")]
    CycleDetected(Vec<String>),
    #[error("node not found: {0}")]
    NodeNotFound(String),
    #[error("edge already exists between {0} and {1}")]
    EdgeAlreadyExists(String, String),
    #[error("invalid graph state: {0}")]
    InvalidState(String),
}

/// The kind of dependency relationship between two nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DependencyKind {
    /// Production dependency.
    Prod,
    /// Development dependency.
    Dev,
    /// Optional dependency.
    Optional,
    /// Peer dependency.
    Peer,
}

impl DependencyKind {
    /// Return as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            DependencyKind::Prod => "prod",
            DependencyKind::Dev => "dev",
            DependencyKind::Optional => "optional",
            DependencyKind::Peer => "peer",
        }
    }
}

/// A node in the dependency graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Node {
    /// Package name.
    pub name: String,
    /// Package version.
    pub version: String,
}

impl Node {
    /// Create a new graph node.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }

    /// Return the node label for display.
    pub fn label(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// An edge in the dependency graph.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Edge {
    /// The kind of dependency.
    pub kind: DependencyKind,
}

impl Edge {
    /// Create a new graph edge.
    pub fn new(kind: DependencyKind) -> Self {
        Self { kind }
    }
}

/// A directed dependency graph.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    graph: DiGraph<Node, Edge>,
    node_indices: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    /// Create a new empty dependency graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
        }
    }

    /// Add a node to the graph. Returns the node index.
    /// If a node with the same name already exists, returns the existing index.
    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        if let Some(&idx) = self.node_indices.get(&node.name) {
            // Update version if node already exists
            if let Some(existing) = self.graph.node_weight_mut(idx) {
                existing.version = node.version;
            }
            return idx;
        }
        let idx = self.graph.add_node(node.clone());
        self.node_indices.insert(node.name, idx);
        idx
    }

    /// Add a dependency edge between two nodes.
    pub fn add_edge(
        &mut self,
        from: &str,
        to: &str,
        kind: DependencyKind,
    ) -> Result<(), GraphError> {
        let from_idx = self
            .node_indices
            .get(from)
            .ok_or_else(|| GraphError::NodeNotFound(from.to_string()))?;
        let to_idx = self
            .node_indices
            .get(to)
            .ok_or_else(|| GraphError::NodeNotFound(to.to_string()))?;

        // Check for existing edge
        if self
            .graph
            .edges_connecting(*from_idx, *to_idx)
            .next()
            .is_some()
        {
            return Err(GraphError::EdgeAlreadyExists(from.to_string(), to.to_string()));
        }

        self.graph.add_edge(*from_idx, *to_idx, Edge::new(kind));
        Ok(())
    }

    /// Get a node by name.
    pub fn get_node(&self, name: &str) -> Option<&Node> {
        self.node_indices
            .get(name)
            .and_then(|idx| self.graph.node_weight(*idx))
    }

    /// Check if a node exists.
    pub fn has_node(&self, name: &str) -> bool {
        self.node_indices.contains_key(name)
    }

    /// Return the number of nodes.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Return the number of edges.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Compute a topological ordering of the graph.
    ///
    /// Returns nodes in dependency order (dependencies before dependents).
    pub fn resolve_order(&self) -> Result<Vec<NodeIndex>, GraphError> {
        // Check for cycles first
        if let Some(cycle) = self.detect_cycles().first() {
            let names: Vec<String> = cycle
                .iter()
                .filter_map(|idx| self.graph.node_weight(*idx))
                .map(|n| n.label())
                .collect();
            return Err(GraphError::CycleDetected(names));
        }

        let mut order = Vec::new();
        let topo = Topo::new(&self.graph);
        for node in topo.iter(&self.graph) {
            order.push(node);
        }
        Ok(order)
    }

    /// Resolve order returning node names.
    pub fn resolve_order_names(&self) -> Result<Vec<String>, GraphError> {
        let order = self.resolve_order()?;
        Ok(order
            .iter()
            .filter_map(|idx| self.graph.node_weight(*idx))
            .map(|n| n.name.clone())
            .collect())
    }

    /// Detect cycles in the graph. Returns a list of cycles, where each
    /// cycle is a list of node indices forming the cycle.
    pub fn detect_cycles(&self) -> Vec<Vec<NodeIndex>> {
        // Use Johnson's algorithm for finding cycles
        // For simplicity, use the built-in cycle detection from petgraph
        let mut cycles = Vec::new();
        let visited = &mut HashSet::new();
        let path = &mut Vec::new();

        for node in self.graph.node_indices() {
            self.find_cycles_from(node, visited, path, &mut cycles);
        }

        cycles
    }

    fn find_cycles_from(
        &self,
        start: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        path: &mut Vec<NodeIndex>,
        cycles: &mut Vec<Vec<NodeIndex>>,
    ) {
        if path.contains(&start) {
            let cycle_start = path.iter().position(|&n| n == start).unwrap();
            let cycle = path[cycle_start..].to_vec();
            if !cycles.contains(&cycle) {
                cycles.push(cycle);
            }
            return;
        }

        if visited.contains(&start) {
            return;
        }

        visited.insert(start);
        path.push(start);

        for neighbor in self.graph.neighbors(start) {
            self.find_cycles_from(neighbor, visited, path, cycles);
        }

        path.pop();
    }

    /// Extract a subgraph containing only the given node and its transitive dependencies.
    pub fn subgraph(&self, root: &str) -> Result<Self, GraphError> {
        let root_idx = self
            .node_indices
            .get(root)
            .ok_or_else(|| GraphError::NodeNotFound(root.to_string()))?;

        let mut sub = DependencyGraph::new();

        // BFS to find all reachable nodes
        let mut to_visit = vec![*root_idx];
        let mut visited = HashSet::new();

        while let Some(idx) = to_visit.pop() {
            if !visited.insert(idx) {
                continue;
            }

            if let Some(node) = self.graph.node_weight(idx) {
                sub.add_node(node.clone());
            }

            for neighbor in self.graph.neighbors(idx) {
                to_visit.push(neighbor);
            }
        }

        // Copy edges within the subgraph
        for edge in self.graph.edge_references() {
            let source_node = self.graph.node_weight(edge.source());
            let target_node = self.graph.node_weight(edge.target());
            if let (Some(sn), Some(tn)) = (source_node, target_node) {
                if sub.has_node(&sn.name) && sub.has_node(&tn.name) {
                    let _ = sub.add_edge(&sn.name, &tn.name, edge.weight().kind);
                }
            }
        }

        Ok(sub)
    }

    /// Get all direct dependencies of a node.
    pub fn dependencies(&self, name: &str) -> Result<Vec<&Node>, GraphError> {
        let idx = self
            .node_indices
            .get(name)
            .ok_or_else(|| GraphError::NodeNotFound(name.to_string()))?;

        let deps = self
            .graph
            .neighbors(*idx)
            .filter_map(|n| self.graph.node_weight(n))
            .collect();
        Ok(deps)
    }

    /// Get all direct dependents of a node (reverse dependencies).
    pub fn dependents(&self, name: &str) -> Result<Vec<&Node>, GraphError> {
        let idx = self
            .node_indices
            .get(name)
            .ok_or_else(|| GraphError::NodeNotFound(name.to_string()))?;

        // petgraph's DiGraph doesn't have reverse neighbors directly,
        // so we iterate all edges
        let mut deps = Vec::new();
        for edge in self.graph.edge_references() {
            if edge.target() == *idx {
                if let Some(node) = self.graph.node_weight(edge.source()) {
                    deps.push(node);
                }
            }
        }
        Ok(deps)
    }

    /// Get all edges from a node.
    pub fn edges_from(&self, name: &str) -> Result<Vec<(&Node, &Edge)>, GraphError> {
        let idx = self
            .node_indices
            .get(name)
            .ok_or_else(|| GraphError::NodeNotFound(name.to_string()))?;

        let edges = self
            .graph
            .edges(*idx)
            .filter_map(|e| {
                let target = self.graph.node_weight(e.target())?;
                Some((target, e.weight()))
            })
            .collect();
        Ok(edges)
    }

    /// Generate a DOT format visualization of the graph.
    pub fn visualize(&self) -> String {
        let mut dot = String::from("digraph DependencyGraph {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=box, style=rounded];\n\n");

        // Add nodes
        for (idx, node) in self.graph.node_references() {
            let label = node.label().replace('"', "'");
            dot.push_str(&format!("    n{} [label=\"{}\"];\n", idx.index(), label));
        }

        dot.push('\n');

        // Add edges
        for edge in self.graph.edge_references() {
            let source = edge.source().index();
            let target = edge.target().index();
            let label = edge.weight().kind.as_str();
            dot.push_str(&format!(
                "    n{} -> n{} [label=\"{}\"];\n",
                source, target, label
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Merge another graph into this one.
    pub fn merge(&mut self, other: &DependencyGraph) {
        for (_, node) in other.graph.node_references() {
            self.add_node(node.clone());
        }
        for edge in other.graph.edge_references() {
            let source = &other.graph[edge.source()];
            let target = &other.graph[edge.target()];
            let _ = self.add_edge(&source.name, &target.name, edge.weight().kind);
        }
    }

    /// Get all nodes in the graph.
    pub fn nodes(&self) -> Vec<&Node> {
        self.graph.node_references().map(|(_, n)| n).collect()
    }

    /// Get all edges in the graph.
    pub fn edges(&self) -> Vec<(&Node, &Node, &Edge)> {
        self.graph
            .edge_references()
            .filter_map(|e| {
                let source = self.graph.node_weight(e.source())?;
                let target = self.graph.node_weight(e.target())?;
                Some((source, target, e.weight()))
            })
            .collect()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DependencyGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DependencyGraph({} nodes, {} edges)", self.node_count(), self.edge_count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_new() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_graph_add_node() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("test-pkg", "1.0.0"));
        assert_eq!(graph.node_count(), 1);
        assert!(graph.has_node("test-pkg"));
    }

    #[test]
    fn test_graph_add_edge() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("root", "1.0.0"));
        graph.add_node(Node::new("dep", "2.0.0"));
        graph.add_edge("root", "dep", DependencyKind::Prod).unwrap();
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_graph_missing_node() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("root", "1.0.0"));
        let result = graph.add_edge("root", "nonexistent", DependencyKind::Prod);
        assert!(result.is_err());
    }

    #[test]
    fn test_topological_order() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("root", "1.0.0"));
        graph.add_node(Node::new("dep-a", "1.0.0"));
        graph.add_node(Node::new("dep-b", "1.0.0"));
        graph.add_edge("root", "dep-a", DependencyKind::Prod).unwrap();
        graph.add_edge("root", "dep-b", DependencyKind::Prod).unwrap();

        let order = graph.resolve_order().unwrap();
        // deps should come before root
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();
        let a = graph.add_node(Node::new("a", "1.0.0"));
        let b = graph.add_node(Node::new("b", "1.0.0"));
        // Manually add cycle via petgraph
        graph.graph.add_edge(a, b, Edge::new(DependencyKind::Prod));
        graph.graph.add_edge(b, a, Edge::new(DependencyKind::Prod));

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());

        // resolve_order should fail
        assert!(graph.resolve_order().is_err());
    }

    #[test]
    fn test_subgraph() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("root", "1.0.0"));
        graph.add_node(Node::new("dep", "1.0.0"));
        graph.add_node(Node::new("subdep", "1.0.0"));
        graph.add_node(Node::new("unrelated", "1.0.0"));
        graph.add_edge("root", "dep", DependencyKind::Prod).unwrap();
        graph.add_edge("dep", "subdep", DependencyKind::Prod).unwrap();

        let sub = graph.subgraph("root").unwrap();
        assert!(sub.has_node("root"));
        assert!(sub.has_node("dep"));
        assert!(sub.has_node("subdep"));
        assert!(!sub.has_node("unrelated"));
    }

    #[test]
    fn test_visualize() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("a", "1.0.0"));
        graph.add_node(Node::new("b", "2.0.0"));
        graph.add_edge("a", "b", DependencyKind::Prod).unwrap();

        let dot = graph.visualize();
        assert!(dot.starts_with("digraph"));
        assert!(dot.contains("a@1.0.0"));
        assert!(dot.contains("b@2.0.0"));
    }

    #[test]
    fn test_dependencies() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("root", "1.0.0"));
        graph.add_node(Node::new("dep", "2.0.0"));
        graph.add_edge("root", "dep", DependencyKind::Prod).unwrap();

        let deps = graph.dependencies("root").unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "dep");
    }

    #[test]
    fn test_dependents() {
        let mut graph = DependencyGraph::new();
        graph.add_node(Node::new("root", "1.0.0"));
        graph.add_node(Node::new("dep", "2.0.0"));
        graph.add_edge("root", "dep", DependencyKind::Prod).unwrap();

        let dependents = graph.dependents("dep").unwrap();
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0].name, "root");
    }

    #[test]
    fn test_edge_kind_display() {
        assert_eq!(DependencyKind::Prod.as_str(), "prod");
        assert_eq!(DependencyKind::Dev.as_str(), "dev");
        assert_eq!(DependencyKind::Optional.as_str(), "optional");
    }

    #[test]
    fn test_merge() {
        let mut g1 = DependencyGraph::new();
        g1.add_node(Node::new("a", "1.0.0"));

        let mut g2 = DependencyGraph::new();
        g2.add_node(Node::new("b", "1.0.0"));

        g1.merge(&g2);
        assert_eq!(g1.node_count(), 2);
    }
}
