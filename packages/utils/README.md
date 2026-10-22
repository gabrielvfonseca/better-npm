# `@better-npm/utils`

General-purpose JavaScript/TypeScript utility functions.

## Overview

Utilities for Bnpm applications: `cn()` (className merging), `isBrowser()`/`isServer()`, `formatBytes()`, `formatDate()`, `formatRelativeTime()`, `truncate()`, `capitalize()`, `pluralize()`, `range()`, `noop()`, `clamp()`, `isObject()`.

## Usage

```ts
import { cn, formatBytes, formatRelativeTime, truncate } from "@better-npm/utils";

cn("px-4", false && "hidden", "py-2"); // "px-4 py-2"
formatBytes(1_500_000); // "1.43 MB"
formatRelativeTime(new Date(Date.now() - 3600000)); // "1 hour ago"
truncate("Hello world", 5); // "He..."
```
