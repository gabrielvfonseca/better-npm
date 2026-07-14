# dash-graph

Dependency graph data structures and traversal algorithms.

## Overview

Builds on petgraph to provide a directed dependency graph with support for cycle detection, topological ordering, subgraph extraction, and visualization.

## Features

- Directed acyclic dependency graph (DAG)
- Cycle detection in dependency trees
- Topological sort for install ordering
- Subgraph extraction for focused operations
- DOT format graph export

## Usage

```rust
use dash_graph::{DependencyGraph, GraphBuilder};

let mut graph = GraphBuilder::new();
graph.add_dep("app", "react", "^18.0.0")?;
graph.add_dep("app", "lodash", "^4.0.0")?;
let order = graph.topological_sort()?;
```
