# bnpm-cli

Bnpm package manager CLI — main binary entrypoint.

## Overview

Provides the CLI entrypoint for Bnpm, including subcommand dispatch, configuration loading, and exit code handling.

## Commands

- `install` — Install packages from the registry
- `publish` — Publish packages to the registry
- `resolve` — Resolve dependency trees
- `cache` — Manage the local cache
- `config` — View and modify configuration

## Usage

```bash
bnpm install <package>
bnpm publish --path ./my-package
bnpm resolve --check
```
