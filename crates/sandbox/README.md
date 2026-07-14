# dash-sandbox

Sandboxed execution environments for package build scripts.

## Overview

Provides isolated execution contexts for running package install scripts with resource limits and filesystem restrictions.

## Features

- Filesystem isolation (chroot-like)
- Resource limits (CPU, memory, timeouts)
- Process-level sandboxing
- Temporary working directories
- Configurable allow/deny permissions

## Usage

```rust
use dash_sandbox::{Sandbox, SandboxConfig};

let sandbox = Sandbox::new(SandboxConfig {
    memory_limit: Some(512 * 1024 * 1024),
    timeout: Some(Duration::from_secs(60)),
    ..Default::default()
})?;
sandbox.run_script("./build.sh")?;
```
