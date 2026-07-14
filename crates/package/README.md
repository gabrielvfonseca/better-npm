# dash-package

Core package model: name, version, metadata, and identifier types.

## Overview

Defines the fundamental types used throughout Dash for representing packages, their identifiers, distribution metadata, and versioning.

## Features

- `PackageId`, `PackageName`, `Version` types
- Semantic versioning support (via `semver`)
- Package metadata structure
- Distribution channel and tag support
- Serde serialization for all types

## Usage

```rust
use dash_package::{PackageId, VersionReq};

let id = PackageId::new("react", "18.2.0")?;
let req = VersionReq::parse("^18.0.0")?;
assert!(req.matches(&id.version()));
```
