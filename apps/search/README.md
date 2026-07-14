# `@dash/apps-search`

Search service — Hono-based search API for package discovery.

## Overview

Hono-based search service wrapping `@dash/search` with HTTP endpoints for full-text search, suggestions, and index management.

## Endpoints

- `GET /search?q=...` — Full-text search
- `GET /suggest?q=...` — Search suggestions
- `POST /index` — Index a package
- `DELETE /index/:name` — Remove from index

## Usage

```ts
import { createSearchService } from "@dash/apps-search";
import { createSearchClient } from "@dash/search";

const client = createSearchClient({ endpoint: "http://meilisearch:7700" });
const app = createSearchService(client);
```
