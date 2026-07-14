# dash-manifest

Package manifest (dash.json) schema, parsing, validation, and canonical CBOR encoding.

## Overview

Handles reading, writing, and validating package manifests (package.json, dash.json) which define package metadata, dependencies, entry points, and build configuration.

## Features

- Manifest parsing (JSON, CBOR)
- Schema validation with `schemars`
- Dependency field extraction
- Canonical CBOR encoding for registry publishing
- Support for workspaces and monorepos

## Usage

```rust
use dash_manifest::Manifest;

let manifest = Manifest::from_path("dash.json")?;
println!("Name: {}", manifest.name());
println!("Dependencies: {:?}", manifest.dependencies());
```
