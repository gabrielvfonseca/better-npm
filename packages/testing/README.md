# `@better-npm/testing`

Test utilities: mock server, test fixtures, temp directories.

## Overview

Helpers for writing Bnpm integration tests: `createTestLogger()`, `createTestConfig()`, `createMockServer()` (Bun-based mock HTTP), `createTestPackage()`, `withTempDir()` (auto-cleaned), and `sleep()`/`randomId()`.

## Usage

```ts
import { createMockServer, withTempDir, createTestPackage } from "@better-npm/testing";

using server = await createMockServer(3001);
await withTempDir(async (dir) => {
  const pkg = createTestPackage({ name: "test-pkg" });
});
```
