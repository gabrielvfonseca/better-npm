# dash-testing

Integration test utilities: test fixtures, mock registries, helpers.

## Overview

Provides building blocks for writing comprehensive tests for Dash packages and subsystems.

## Features

- Test fixture creation and management
- Mock HTTP registry server
- Test database containers (via `testcontainers`)
- Temporary directory and environment setup
- Common assertion helpers

## Usage

```rust
use dash_testing::{TestEnv, MockRegistry};

let mut env = TestEnv::new()?;
let registry = MockRegistry::start().await?;
env.with_registry(registry.url());
```
