# dash-cli

Dash package manager CLI — main binary entrypoint.

## Overview

Provides the CLI entrypoint for Dash, including subcommand dispatch, configuration loading, and exit code handling.

## Commands

- `install` — Install packages from the registry
- `publish` — Publish packages to the registry
- `resolve` — Resolve dependency trees
- `cache` — Manage the local cache
- `config` — View and modify configuration

## Usage

```bash
dash install <package>
dash publish --path ./my-package
dash resolve --check
```
