# dash-network

HTTP networking layer: retry, backoff, rate limiting, middleware.

## Overview

Provides utilities for URL handling, HTTP methods, retry logic, and default headers for Dash's registry communication.

## Features

- Configurable retry with exponential backoff
- Rate limiting for API requests
- Reqwest-based HTTP client wrapper
- Custom headers and middleware
- Connection pooling

## Usage

```rust
use dash_network::{HttpClient, HttpClientConfig};

let client = HttpClient::new(HttpClientConfig {
    retry_limit: 3,
    ..Default::default()
})?;
let response = client.get("https://registry.dash.sh/pkg/react").await?;
```
