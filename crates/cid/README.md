# bnpm-cid

BLAKE3 CID, multihash, and base32 canonical encoding.

## Overview

Content Identifiers (CIDs) based on BLAKE3 multihash with canonical base32 encoding for Bnpm packages.

## Features

- BLAKE3-based content addressing
- Multihash format support
- Canonical base32 encoding (no padding)
- CID parsing and validation
- Serde serialization support

## Usage

```rust
use bnpm_cid::Cid;

let cid = Cid::new(b"hello world");
assert_eq!(cid, Cid::parse(&cid.to_string())?);
```
