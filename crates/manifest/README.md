# bnpm-manifest

Package manifest (bnpm.json) schema, parsing, validation, and canonical CBOR encoding.

## Overview

Handles reading, writing, and validating package manifests (package.json, bnpm.json) which define package metadata, dependencies, entry points, and build configuration.

## Features

- Manifest parsing (JSON, CBOR)
- Schema validation with `schemars`
- Dependency field extraction
- Canonical CBOR encoding for registry publishing
- Support for workspaces and monorepos

## Usage

```rust
use bnpm_manifest::Manifest;

let manifest = Manifest::from_path("bnpm.json")?;
println!("Name: {}", manifest.name());
println!("Dependencies: {:?}", manifest.dependencies());
```
