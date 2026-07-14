# `@dash/cache`

Multi-tier caching with in-memory and Redis backends, plus memoization.

## Overview

Provides a `CacheBackend` interface with `MemoryCache` (TTL-based, auto-cleanup) and `RedisCache` implementations, a `createCache()` factory, and a `memoize()` function for caching async function results.

## Usage

```ts
import { MemoryCache, memoize } from "@dash/cache";

const cache = new MemoryCache<string>({ ttl: 300_000 });
await cache.set("key", "value");
const value = await cache.get("key");

const cachedFn = memoize(myExpensiveFn, { ttl: 60_000 });
```
