# bnpm-protocol

Wire protocol types and serialization for registry communication.

## Overview

Defines the message format used between Bnpm clients and registries, including framing, message types, and serialization helpers.

## Features

- Registry API request/response types
- Message framing and serialization (JSON + binary)
- gRPC service definitions (via `prost`/`tonic`)
- Pagination and query parameter types

## Usage

```rust
use bnpm_protocol::{PackageMetadata, VersionInfo};

let metadata = registry_client.get_package("react").await?;
println!("Latest: {}", metadata.latest_version());
```
