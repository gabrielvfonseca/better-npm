# `@dash/workers-metadata`

Metadata enrichment worker.

## Overview

Extracts and enriches package metadata: parses GitHub repository URLs, resolves SPDX license identifiers, normalizes keywords, builds dependency graphs, and computes readme statistics.

## Usage

```ts
import { handler } from "@dash/workers-metadata";

const result = await handler({ name: "react", version: "18.2.0", repository: { url: "https://github.com/facebook/react" } });
```
