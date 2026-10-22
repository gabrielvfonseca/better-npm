# bnpm-signatures

Cryptographic signature generation and verification (Ed25519 + RSA).

## Overview

Provides high-level APIs for signing package payloads and verifying signatures against trusted public keys.

## Features

- Ed25519 signature generation and verification
- RSA-PSS signature support
- Package payload signing
- Public key management and trust stores
- Signature envelope format

## Usage

```rust
use bnpm_signatures::{Signer, Verifier, KeyPair};

let keypair = KeyPair::generate_ed25519()?;
let signature = Signer::sign(b"package data", &keypair.secret)?;
assert!(Verifier::verify(b"package data", &signature, &keypair.public)?);
```
