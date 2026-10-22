# `@better-npm/search`

Search client for MeiliSearch-compatible registry search.

## Overview

Provides a `SearchClient` with methods for `search`, `suggest`, `index`, and `remove` operations against a search service API, plus `SearchQuery`, `SearchResult`, `SearchSuggestion`, and `SearchConfig` types.

## Usage

```ts
import { createSearchClient } from "@better-npm/search";

const client = createSearchClient({ endpoint: "https://search.bnpm.sh" });
const results = await client.search({ q: "react", limit: 20 });
const suggestions = await client.suggest("rea");
```
