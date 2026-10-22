# bnpm-installer

Package installation orchestration with parallel extraction and verification.

## Overview

Coordinates the full install pipeline: resolving dependencies, downloading packages, verifying integrity, extracting archives, and linking binaries.

## Features

- Full install pipeline orchestration
- Parallel package extraction
- Integrity verification during install
- Transactional rollback on failure
- Integration with sandbox for script execution

## Usage

```rust
use bnpm_installer::Installer;

let installer = Installer::new(&config)?;
let result = installer
    .install(&package_spec)
    .with_progress(true)
    .await?;
```
