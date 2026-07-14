# dash-compression

Compression and decompression support (gzip, zstd, deflate).

## Overview

Provides helpers for compressing and decompressing byte buffers and files using various algorithms.

## Features

- gzip compression/decompression
- zstd compression/decompression
- deflate compression/decompression
- Streaming and buffer-based APIs
- Async file compression

## Usage

```rust
use dash_compression::{compress, decompress, CompressionFormat};

let compressed = compress(b"data", CompressionFormat::Zstd, 3)?;
let original = decompress(&compressed, CompressionFormat::Zstd)?;
```
