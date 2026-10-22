# `@better-npm/storage`

Storage abstraction with local filesystem and S3-compatible backends.

## Overview

Defines a `StorageBackend` interface with `LocalStorage` (filesystem-based, directory traversal protection) and `S3Storage` (S3-compatible HTTP API, Basic auth) implementations. Includes a `StorageFactory` and `getStorage()` factory.

## Usage

```ts
import { getStorage } from "@better-npm/storage";

const storage = getStorage({ type: "local", basePath: "/data/bnpm" });
await storage.write("packages/react.tgz", buffer);
const data = await storage.read("packages/react.tgz");
```
