# `@dash/sdk`

Unified Dash SDK — single entry point for client, logger, config, auth, schemas, and utilities.

## Overview

Combines `@dash/client`, `@dash/types`, `@dash/logger`, `@dash/config`, `@dash/auth`, and `@dash/schemas` into a single `Dash` class with a `createDash()` factory.

## Usage

```ts
import { createDash } from "@dash/sdk";

const dash = await createDash({ token: "dash_tkn_xxx" });
const pkg = await dash.client.get("/packages/react");
dash.log.info("Fetched package", { name: pkg.name });
```
