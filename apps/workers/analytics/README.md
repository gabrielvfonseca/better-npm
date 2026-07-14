# `@dash/workers-analytics`

Analytics event processing worker.

## Overview

Processes `AnalyticsPayload` events by enriching them with metadata and ingesting via the telemetry client. Supports optional event aggregation and batching.

## Usage

```ts
import { handler } from "@dash/workers-analytics";

await handler({ events: [{ type: "page_view", properties: { path: "/" } }] });
```
