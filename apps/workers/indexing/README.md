# `@dash/workers-indexing`

Search indexing worker.

## Overview

Handles `IndexPayload` documents to index, reindex, or remove package documents from the search service. Constructs search documents from package metadata.

## Usage

```ts
import { handler } from "@dash/workers-indexing";

await handler({ action: "index", name: "react", version: "18.2.0", metadata: { description: "A JavaScript library" } });
```
