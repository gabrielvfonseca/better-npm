# bnpm-store

Persistent and object storage abstraction with local and cloud backends.

## Overview

Provides a content-addressable store where data is keyed by its hash, supporting local filesystem and object store (S3, GCS, Azure) backends.

## Features

- Content-addressable storage (CAS)
- Local filesystem backend
- S3-compatible object store backend
- GCS and Azure Blob backend support
- Caching layer integrated with `bnpm-cache`
- Concurrent read/write access

## Usage

```rust
use bnpm_store::{Store, StoreConfig};

let store = Store::new(StoreConfig::default())?;
let cid = store.put(b"package data")?;
let data = store.get(&cid)?;
```
