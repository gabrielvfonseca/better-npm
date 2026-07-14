# `@dash/workers-cleanup`

Stale data cleanup worker.

## Overview

Removes stale data from the system: expired cache entries, temporary files, old package versions, rotated logs, and orphaned metadata entries based on cutoff dates.

## Usage

```ts
import { handler } from "@dash/workers-cleanup";

await handler({ types: ["cache", "temp", "old-versions", "logs", "metadata"] });
```
