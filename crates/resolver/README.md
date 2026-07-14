# dash-resolver

Dependency resolution algorithm for Dash package graphs.

## Overview

Implements Minimal Version Selection (MVS) algorithm from Russ Cox / Go modules, with PubGrub fallback for peer dependency conflicts.

## Features

- Minimal Version Selection (MVS) algorithm
- PubGrub fallback for conflict resolution
- Peer dependency handling
- Lockfile-aware resolution
- Caching of resolved subgraphs

## Usage

```rust
use dash_resolver::{Resolver, ResolveConfig};

let resolver = Resolver::new(&config)?;
let solution = resolver
    .add_root("my-app", "1.0.0")
    .resolve()
    .await?;
```
