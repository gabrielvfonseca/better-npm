# bnpm-config

Configuration management: layered profiles, env overrides, CLI args.

## Overview

Bnpm loads configuration from multiple sources in order of increasing priority: defaults → config files → environment variables → CLI args.

## Features

- Layered configuration merging
- Profile support (dev, production, custom)
- Environment variable overrides
- JSON, YAML, and TOML config file formats
- Schema generation with `schemars`

## Usage

```rust
use bnpm_config::{Config, ConfigBuilder};

let config = ConfigBuilder::new()
    .with_profile("production")
    .with_cli_overrides(matches)
    .load()?;
```
