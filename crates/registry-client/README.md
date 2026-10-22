# bnpm-registry-client

HTTP client for communicating with Bnpm package registries.

## Overview

Provides methods for fetching package metadata, searching, publishing packages, and managing authentication with registries.

## Features

- Package metadata and version lookup
- Search and query endpoints
- Package publish and unpublish
- Authentication token management
- Pagination support for large result sets

## Usage

```rust
use bnpm_registry_client::RegistryClient;

let client = RegistryClient::new("https://registry.bnpm.sh")?;
let pkg = client.get_package("react").await?;
let versions = client.list_versions("react").await?;
```
