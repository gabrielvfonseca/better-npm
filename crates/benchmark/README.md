# dash-benchmark

Benchmarking harness using Criterion for Dash subsystems.

## Overview

Provides a simple API for running benchmarks and collecting performance statistics on sync and async operations.

## Features

- Criterion-based benchmarking harness
- Support for sync and async benchmarks
- Performance statistics collection
- Configurable benchmark parameters

## Usage

```rust
use dash_benchmark::{Benchmark, BenchmarkConfig};

let mut bench = Benchmark::new("resolve_deps", config);
bench.run(|| {
    resolver.resolve(&dependencies)?;
})?;
```
