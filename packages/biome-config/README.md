# `@better-npm/biome-config`

Shared Biome configuration for the Bnpm monorepo.

## Overview

Exports a strict Biome configuration with lint rules across all categories (complexity, correctness, performance, security, style, suspicious, a11y) and formatter settings (spaces, double quotes, trailing commas).

## Usage

In `biome.json`:

```json
{
  "extends": ["@better-npm/biome-config/biome.json"]
}
```
