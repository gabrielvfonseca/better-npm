# `@better-npm/apps-telemetry`

Telemetry ingestion service — Hono-based events and metrics API.

## Overview

Hono-based service for ingesting analytics events and performance metrics. Accepts batch event/metric payloads and exposes a Prometheus-format metrics endpoint.

## Endpoints

- `POST /events` — Batch event ingestion (max 1000)
- `POST /metrics` — Batch metric ingestion (max 5000)
- `GET /metrics` — Prometheus metrics format
- `GET /health` — Health check

## Usage

```ts
import { createTelemetryService } from "@better-npm/apps-telemetry";
import { createTelemetryClient } from "@better-npm/telemetry";

const client = createTelemetryClient({ endpoint: "http://localhost:4318" });
const app = createTelemetryService(client);
```
