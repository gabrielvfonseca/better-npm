# `@better-npm/sdk`

Unified Bnpm SDK — single entry point for client, logger, config, auth, schemas, and utilities.

## Overview

Combines `@better-npm/client`, `@better-npm/types`, `@better-npm/logger`, `@better-npm/config`, `@better-npm/auth`, and `@better-npm/schemas` into a single `Bnpm` class with a `createBnpm()` factory.

## Usage

```ts
import { createBnpm } from "@better-npm/sdk";

const bnpm = await createBnpm({ token: "bnpm_tkn_xxx" });
const pkg = await bnpm.client.get("/packages/react");
bnpm.log.info("Fetched package", { name: pkg.name });
```
