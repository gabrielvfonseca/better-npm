# bnpm-runtime

Application runtime initialization and lifecycle management.

## Overview

Provides a unified runtime that sets up tokio, logging, telemetry, and configuration, and manages graceful shutdown.

## Features

- Tokio async runtime initialization
- Logging and telemetry setup
- Configuration loading
- Graceful shutdown with signal handling
- Resource cleanup on exit

## Usage

```rust
use bnpm_runtime::{Runtime, RuntimeConfig};

let rt = Runtime::new(RuntimeConfig::default())?;
rt.run(async {
    // Application logic here
}).await?;
```
