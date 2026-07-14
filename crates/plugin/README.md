# dash-plugin

Dynamic plugin system for extending Dash via shared libraries.

## Overview

Provides a trait-based plugin interface and a manager for loading, unloading, and accessing plugins at runtime.

## Features

- Loadable shared library plugins (via `libloading`)
- Trait-based plugin interface
- Plugin lifecycle management (load, unload, reload)
- Plugin configuration and discovery

## Usage

```rust
use dash_plugin::{PluginManager, Plugin};

let mut manager = PluginManager::new(&config)?;
let plugin = manager.load("my-plugin")?;
plugin.initialize()?;
```
