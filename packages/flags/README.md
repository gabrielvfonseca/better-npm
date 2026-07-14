# `@dash/flags`

Feature flag evaluation with context-based rules.

## Overview

Evaluates feature flags against a context object using rules (`eq`, `neq`, `in`, `nin`, `gt`, `gte`, `lt`, `lte`, `contains`). Supports flag name, rule, and type definitions.

## Usage

```ts
import { createFeatureFlags } from "@dash/flags";

const flags = createFeatureFlags({
  "new-installer": { rules: [{ field: "userId", operator: "gte", value: 1000 }] },
});

const enabled = flags.evaluate("new-installer", { userId: 5000 });
```
