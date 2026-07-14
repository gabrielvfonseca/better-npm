# dash-cache

Multi-tier caching layer with in-memory (Moka) and optional Redis backends.

## Overview

Provides a unified `CacheBackend` trait with interchangeable implementations for local and distributed caching.

## Features

- In-memory caching with Moka (concurrent, high-performance)
- Optional Redis-backed distributed cache
- Unified `CacheBackend` trait for interchangeable backends
- TTL-based expiration and cache invalidation

## Usage

```rust
use dash_cache::{CacheBackend, MemoryCache};

let cache = MemoryCache::new(10_000);
cache.set("key", value, Some(Duration::from_secs(300)))?;
let cached: Option<Value> = cache.get("key")?;
```
