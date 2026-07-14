# dash-publisher

Package publishing workflow: build, sign, upload to registry.

## Overview

Orchestrates the complete publish pipeline: preparing the package tarball, computing integrity hashes, signing, and uploading.

## Features

- Package tarball preparation and compression
- Integrity hash computation
- Cryptographic signing (via `dash-signatures`)
- Registry upload with authentication
- Dry-run and validation modes

## Usage

```rust
use dash_publisher::Publisher;

let publisher = Publisher::new(&config)?;
let result = publisher
    .publish("./my-package")
    .with_signature(true)
    .await?;
```
