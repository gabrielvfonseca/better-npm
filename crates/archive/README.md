# bnpm-archive

Package archive creation and extraction (tar/zip with optional compression).

## Overview

Provides a unified interface for creating and extracting package archives in various formats.

## Features

- Tar and Zip archive creation and extraction
- Optional compression (gzip, zstd) integration
- Streaming archive processing
- Integration with `bnpm-filesystem` and `bnpm-compression`

## Usage

```rust
use bnpm_archive::{ArchiveBuilder, ArchiveFormat};

// Create a new archive
let archive = ArchiveBuilder::new(ArchiveFormat::TarGz)
    .add_file("path/to/file.txt")?
    .build(&output_path)?;

// Extract an archive
archive.extract_to(&destination_path)?;
```
