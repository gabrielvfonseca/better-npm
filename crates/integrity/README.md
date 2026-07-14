# dash-integrity

Content hash verification and integrity checking for packages.

## Overview

Defines the `Integrity` type used throughout Dash for verifying that downloaded package contents match their expected hashes.

## Features

- `Integrity` type with multiple hash algorithm support
- Content verification against expected hashes
- File-level and stream-level verification
- Integration with SRI (Subresource Integrity) format

## Usage

```rust
use dash_integrity::{Integrity, IntegrityCheck};

let expected = Integrity::parse("sha256-abc123...")?;
let check = IntegrityCheck::new(&expected);
assert!(check.verify(b"actual content")?);
```
