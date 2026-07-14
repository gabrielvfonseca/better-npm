# `@dash/contracts`

Type-safe API route contracts for the Dash registry.

## Overview

Defines `ApiContract` interface and `createContractRoute()` helper with predefined contracts for all registry API endpoints: get/list/publish/deprecate/unpublish packages, search, suggestions, auth, profile, downloads, and stats.

## Usage

```ts
import { getPackage, publishPackage } from "@dash/contracts";

const route = getPackage({ name: "react" });
const publish = publishPackage({ name: "my-pkg", version: "1.0.0" });
```
