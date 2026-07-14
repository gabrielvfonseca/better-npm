# dash-concurrency

Async and parallel concurrency primitives for Dash workflows.

## Overview

Provides helpers for bounded parallelism, task groups, rate limiting, and parallel map/filter patterns.

## Features

- Bounded parallel execution (semaphore-based)
- Async task groups with error handling
- Rate limiting for API calls
- Parallel map/filter on iterators
- Rayon-based CPU-bound task offloading

## Usage

```rust
use dash_concurrency::{parallel_map, ParConfig};

let results = parallel_map(
    items,
    |item| process_item(item),
    ParConfig::default(),
).await?;
```
