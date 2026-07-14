# `@dash/status`

System status and monitoring service — Hono-based health dashboard.

## Overview

Hono-based monitoring service that checks component health (API, Auth, Registry, Search, Telemetry, optional database) and reports system status, component states, and incident history.

## Endpoints

- `GET /status` — Overall system status
- `GET /status/components` — Individual component statuses
- `GET /status/incidents` — Incident history
- `GET /health` — Health check

## Usage

```ts
import { createStatusService } from "@dash/status";
import { createTelemetryClient } from "@dash/telemetry";

const telemetry = createTelemetryClient({ endpoint: "http://localhost:4318" });
const app = createStatusService(telemetry);
```
