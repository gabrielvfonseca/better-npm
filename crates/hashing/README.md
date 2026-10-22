# bnpm-hashing

Unified hashing interface supporting SHA-2, SHA-3, and BLAKE3.

## Overview

Provides a consistent API for computing hashes across multiple algorithms, with helpers for files, streams, and hex encoding.

## Features

- SHA-224/256/384/512, SHA-3, and BLAKE3 support
- Unified `Hasher` trait
- Streaming hash computation
- Hex and base64 encoding helpers
- File content hashing

## Usage

```rust
use bnpm_hashing::{Hasher, HashAlgorithm};

let hash = Hasher::new(HashAlgorithm::Blake3)
    .update(b"hello")
    .update(b" world")
    .finalize();
println!("{}", hash.to_hex());
```
