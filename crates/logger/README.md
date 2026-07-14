# dash-logger

Structured logging with file rotation, JSON output, and env filtering.

## Overview

Builds on `tracing` and `tracing-subscriber` to provide a flexible logging subsystem with multiple output formats and destinations.

## Features

- Structured JSON log output
- Human-readable text format
- Auto-rotating log files
- Environment-based log level filtering
- Integration with `dash-telemetry`

## Usage

```rust
use dash_logger::{Logger, LogConfig};

let logger = Logger::new(LogConfig::from_env()?)?;
logger.init()?;
tracing::info!("Application started");
```
