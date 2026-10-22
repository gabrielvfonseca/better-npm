# `@better-npm/config`

Centralized configuration loading with env overrides and defaults.

## Overview

Defines the `BnpmConfig` interface covering api, registry, auth, telemetry, storage, cache, workers, search, logging, and benchmarks. Loads config from environment variables and optional JSON files.

## Usage

```ts
import { loadConfig, getConfig, isDevelopment } from "@better-npm/config";

const config = await loadConfig({ profile: "production" });
if (isDevelopment()) {
  console.log("Dev mode");
}
```
