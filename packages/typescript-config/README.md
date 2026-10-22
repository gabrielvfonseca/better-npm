# `@better-npm/typescript-config`

Shared TypeScript configuration for the Bnpm monorepo.

## Overview

Base TypeScript configuration with strict mode, ESNext target/module, bundler module resolution, declarations, source maps, and composite/incremental builds.

## Usage

In `tsconfig.json`:

```json
{
  "extends": "@better-npm/typescript-config/tsconfig.base.json"
}
```
