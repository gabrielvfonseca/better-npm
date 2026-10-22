# bnpm-auth

Authentication and token management for registry access.

## Overview

Supports JWT-based authentication, OAuth2 flows, and basic token-based auth for Bnpm registry operations.

## Features

- JWT token generation and validation
- OAuth2 authorization code and device flows
- Token storage and refresh
- Bearer token management for API requests

## Usage

```rust
use bnpm_auth::{AuthProvider, Credentials};

let provider = AuthProvider::new(registry_url);
let token = provider.authenticate(Credentials::Token("my_token")).await?;
```
