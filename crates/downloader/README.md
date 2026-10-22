# bnpm-downloader

Concurrent HTTP/registry download with streaming integrity verification.

## Overview

Downloads packages from the registry with progress reporting, caching, integrity checking, and parallel batch downloads.

## Features

- Concurrent package downloads
- Streaming integrity verification (BLAKE3/SHA-256)
- Progress reporting via `bnpm-progress`
- Caching with `bnpm-cache`
- Parallel batch download with backpressure

## Usage

```rust
use bnpm_downloader::DownloadClient;

let client = DownloadClient::new(&config)?;
let result = client
    .download(&package_id)
    .with_progress(true)
    .await?;
```
