# `@better-npm/logger`

Structured logging with levels, JSON/text output, and child loggers.

## Overview

Module-scoped logger with `debug`/`info`/`warn`/`error`/`fatal` levels, configurable output format (JSON or text), stdout/stderr routing, and child logger creation.

## Usage

```ts
import { createLogger } from "@better-npm/logger";

const log = createLogger({ name: "registry", level: "info" });
log.info("Server starting", { port: 3000 });
log.error("Failed to connect", { error: err.message });

const child = log.child({ module: "auth" });
```
