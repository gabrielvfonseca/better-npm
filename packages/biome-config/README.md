# `@dash/biome-config`

Shared Biome configuration for the Dash monorepo.

## Overview

Exports a strict Biome configuration with lint rules across all categories (complexity, correctness, performance, security, style, suspicious, a11y) and formatter settings (spaces, double quotes, trailing commas).

## Usage

In `biome.json`:

```json
{
  "extends": ["@dash/biome-config/biome.json"]
}
```
