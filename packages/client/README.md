# `@dash/client`

HTTP client with retry, timeout, and Bearer token authentication.

## Overview

A configurable HTTP client with GET/POST/PUT/DELETE/PATCH methods, exponential backoff retry, request timeouts, and Bearer token auth.

## Usage

```ts
import { createClient } from "@dash/client";

const client = createClient({
  baseUrl: "https://registry.dash.sh",
  token: "dash_tkn_xxx",
});

const pkg = await client.get("/packages/react");
const result = await client.post("/packages", { name: "my-pkg" });
```
