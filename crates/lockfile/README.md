# bnpm-lockfile

Lockfile format parsing, generation, and resolution tracking.

## Overview

The lockfile records the exact resolved versions and integrity hashes for all dependencies, ensuring reproducible installs.

## Features

- Lockfile parsing (JSON, YAML, TOML)
- Lockfile generation after resolution
- Integrity hash tracking per package
- Version pinning for reproducible builds
- Conflict detection on lockfile updates

## Usage

```rust
use bnpm_lockfile::{Lockfile, LockfileBuilder};

let lockfile = LockfileBuilder::new()
    .add_entry("react", "18.2.0", integrity_hash)?
    .build()?;
lockfile.write_to("bnpm.lock")?;
```
