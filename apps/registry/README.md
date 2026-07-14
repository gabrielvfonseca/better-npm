# `@dash/registry`

Package registry service — Hono-based registry API.

## Overview

Hono-based package registry service for hosting and serving Dash packages. Supports package CRUD, search, and deprecation with OpenAPI documentation.

## Endpoints

- `GET /packages/:name` — Get package metadata
- `GET /packages/:name/:version` — Get specific version
- `PUT /packages/:name` — Publish package
- `DELETE /packages/:name/:version` — Unpublish version
- `GET /search?q=...` — Search packages
- `POST /packages/:name/deprecate` — Deprecate package

## Usage

```ts
import { createRegistry } from "@dash/registry";
import { getStorage } from "@dash/storage";

const storage = getStorage({ type: "local", basePath: "./data" });
const app = createRegistry(storage);
```
