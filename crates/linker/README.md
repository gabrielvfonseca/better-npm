# bnpm-linker

Symbolic and hard linking of installed packages into target directories.

## Overview

Manages the creation and removal of binary symlinks, `.bin` entries, and other system-level links for installed packages.

## Features

- Symlink creation for package binaries
- Hard link support for deduplication
- Cross-platform link management
- Cleanup and unlink operations

## Usage

```rust
use bnpm_linker::Linker;

let linker = Linker::new(&config)?;
linker.link_bin("my-package", "my-package-bin")?;
linker.unlink("my-package")?;
```
