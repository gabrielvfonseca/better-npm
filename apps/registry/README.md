# `@better-npm/registry`

Package registry service — Hono-based registry API.

## Overview

Hono-based package registry service for hosting and serving Bnpm packages. Supports package CRUD, search, and deprecation with OpenAPI documentation.

## Endpoints

- `GET /packages/:name` — Get package metadata
- `GET /packages/:name/:version` — Get specific version
- `PUT /packages/:name` — Publish package
- `DELETE /packages/:name/:version` — Unpublish version
- `GET /search?q=...` — Search packages
- `POST /packages/:name/deprecate` — Deprecate package

## Usage

```ts
import { createRegistry } from "@better-npm/registry";
import { getStorage } from "@better-npm/storage";

const storage = getStorage({ type: "local", basePath: "./data" });
const app = createRegistry(storage);
```
