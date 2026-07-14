# `@dash/api`

Type-safe API route definitions, middleware types, and response helpers.

## Overview

Provides core API types (`ApiHandler`, `ApiMiddleware`, `ApiRoute`, `ApiRouter`) and HTTP response helpers (`ok`, `created`, `badRequest`, `notFound`, etc.) used across Dash services.

## Usage

```ts
import { ok, badRequest, ApiHandler } from "@dash/api";

const handler: ApiHandler = async (req, ctx) => {
  if (!req.query.q) return badRequest("Missing query");
  return ok({ results: [] });
};
```
