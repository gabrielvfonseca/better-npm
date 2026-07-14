# `@dash/playground`

Code playground service — Hono-based sandboxed JavaScript execution.

## Overview

Hono-based web service that hosts an HTML UI for writing and executing JavaScript code in a sandboxed `new Function()` environment. Includes rate limiting (30 req/min per IP) and a code formatting endpoint.

## Endpoints

- `GET /playground` — Playground HTML UI
- `POST /playground/run` — Execute JavaScript code (sandboxed)
- `POST /playground/format` — Format code

## Usage

```ts
import { createApp } from "@dash/playground";

const app = createApp({ port: 3002 });
```
