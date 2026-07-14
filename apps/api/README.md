# `@dash/apps-api`

Dash API gateway — Hono-based HTTP API with OpenAPI documentation.

## Overview

Hono application with CORS, request ID logging, error handling, health checks, and Swagger UI documentation. Routes requests via `/api/v1` to internal services.

## Endpoints

- `GET /health` — Health check
- `GET /docs` — Swagger UI
- `/api/v1/*` — API version 1 routes

## Usage

```ts
import { createApp, start } from "@dash/apps-api";

const app = createApp({ port: 3000 });
await start(app);
```
