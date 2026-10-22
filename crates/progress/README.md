# bnpm-progress

Terminal progress bars and spinners for long-running operations.

## Overview

Wraps `indicatif` to provide a consistent progress-reporting API for downloads, installations, and other batch operations.

## Features

- Multi-progress bar support
- Spinner for indeterminate operations
- Download speed and ETA tracking
- Configurable bar styling and templates

## Usage

```rust
use bnpm_progress::{ProgressBar, ProgressStyle};

let bar = ProgressBar::new(total_items);
bar.set_style(ProgressStyle::default());
for item in items {
    process(item)?;
    bar.inc(1);
}
bar.finish();
```
