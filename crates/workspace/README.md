# bnpm-workspace

Workspace-level discovery, configuration, and multi-package operations.

## Overview

Manages a workspace of multiple packages, handling member discovery, filtering, and cross-package dependency resolution.

## Features

- Workspace member discovery (glob-based)
- Workspace root and member configuration
- Cross-package dependency resolution
- Member filtering and sorting
- Workspace-level lockfile management

## Usage

```rust
use bnpm_workspace::Workspace;

let ws = Workspace::discover(".")?;
for member in ws.members() {
    println!("Found package: {}", member.name());
}
```
