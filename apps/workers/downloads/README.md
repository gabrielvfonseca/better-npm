# `@dash/workers-downloads`

Package download worker.

## Overview

Downloads package tarballs from remote URLs, verifies SHA-512 integrity, checks content-length headers, caches to disk, and stores in remote storage.

## Usage

```ts
import { handler } from "@dash/workers-downloads";

await handler({ name: "react", version: "18.2.0", url: "https://registry.dash.sh/packages/react/18.2.0.tgz", integrity: "sha512-..." });
```
