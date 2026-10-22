# `@better-npm/schemas`

Zod validation schemas for Bnpm data models.

## Overview

Zod schemas for package names, semver, manifests, publish requests, search queries, pagination, user auth, API errors, and worker payloads. Exports both schemas and inferred TypeScript types.

## Usage

```ts
import { packageNameSchema, semVerSchema, publishPackageSchema } from "@better-npm/schemas";

const name = packageNameSchema.parse("my-package");
const version = semVerSchema.parse("1.0.0");
const result = publishPackageSchema.safeParse(input);
```
