# bnpm-network

HTTP networking layer: retry, backoff, rate limiting, middleware.

## Overview

Provides utilities for URL handling, HTTP methods, retry logic, and default headers for Bnpm's registry communication.

## Features

- Configurable retry with exponential backoff
- Rate limiting for API requests
- Reqwest-based HTTP client wrapper
- Custom headers and middleware
- Connection pooling

## Usage

```rust
use bnpm_network::{HttpClient, HttpClientConfig};

let client = HttpClient::new(HttpClientConfig {
    retry_limit: 3,
    ..Default::default()
})?;
let response = client.get("https://registry.bnpm.sh/pkg/react").await?;
```
