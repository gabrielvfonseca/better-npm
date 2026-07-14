# `@dash/telemetry`

Telemetry and observability: metrics, tracing, and OpenTelemetry integration.

## Overview

Provides a `TelemetryClient` for event/metric ingestion, Prometheus export, and tracing utilities (`startTracing`, `shutdownTracing`, `wrapWithSpan`). Includes metric recording functions (`recordMetric`, `incrementCounter`, `recordHistogram`, `setAttribute`).

## Usage

```ts
import { createTelemetryClient, recordMetric, wrapWithSpan } from "@dash/telemetry";

const telemetry = createTelemetryClient({ endpoint: "https://telemetry.dash.sh" });
await telemetry.ingestEvent({ type: "package_install", properties: { name: "react" } });

recordMetric("packages_downloaded", 42);
```
