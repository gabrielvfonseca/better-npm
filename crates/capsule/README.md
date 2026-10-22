# bnpm-capsule

BNPMCAP1 capsule pack/unpack, file table, and range read.

## Overview

Implements the BNPMCAP1 capsule format for efficient package storage and retrieval. Supports structured file tables and byte-range reads.

## Features

- Capsule packing (encoding) and unpacking (decoding)
- File table metadata indexed within capsules
- Byte-range random access reads
- CBOR-based serialization with optional zstd compression

## Usage

```rust
use bnpm_capsule::{Capsule, CapsuleBuilder};

let capsule = CapsuleBuilder::new()
    .add_file("index.js", &bytes1)
    .add_file("package.json", &bytes2)
    .build()?;

let extracted = capsule.read_file("index.js")?;
```
