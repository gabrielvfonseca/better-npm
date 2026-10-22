# bnpm-crypto

Cryptographic primitives and key management for Bnpm.

## Overview

Provides key generation, symmetric encryption/decryption, and re-exports of common hash and signature algorithms.

## Features

- Key generation (RSA, Ed25519, AES)
- Symmetric encryption/decryption (AES-GCM)
- Re-export of SHA-2, SHA-3, and BLAKE3
- Zeroizing memory of sensitive key material

## Usage

```rust
use bnpm_crypto::{KeyPair, encrypt, decrypt};

let keypair = KeyPair::generate_ed25519()?;
let ciphertext = encrypt(b"secret data", &keypair.secret)?;
let plaintext = decrypt(&ciphertext, &keypair.secret)?;
```
