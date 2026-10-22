# bnpm-diagnostics

Error diagnostics, reporting, and structured diagnostic data.

## Overview

Provides infrastructure for collecting system information, reporting diagnostic issues, and suggesting remediation steps.

## Features

- System information collection
- Structured diagnostic reports
- Error remediation suggestions
- Diagnostic data export

## Usage

```rust
use bnpm_diagnostics::{Diagnostics, Report};

let diag = Diagnostics::new();
let report: Report = diag.collect()?;
println!("{}", report.render());
```
