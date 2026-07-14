# dash-telemetry

Telemetry and observability: tracing, metrics, OpenTelemetry export.

## Overview

Initializes and manages the tracing infrastructure with optional export to OpenTelemetry-compatible backends.

## Features

- Distributed tracing with `tracing` crate
- Metrics collection
- OpenTelemetry OTLP export
- Span and event instrumentation
- Configurable sampling

## Usage

```rust
use dash_telemetry::{Telemetry, TelemetryConfig};

let telemetry = Telemetry::new(TelemetryConfig {
    service_name: "dash".into(),
    otlp_endpoint: Some("http://localhost:4317".into()),
    ..Default::default()
})?;
telemetry.init()?;
```
