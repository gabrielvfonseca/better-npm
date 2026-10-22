# bnpm-provenance

Sigstore/OIDC keyless signing, in-toto/SLSA attestations, transparency log proofs.

## Overview

Provides cryptographic provenance for packages using Sigstore's keyless signing, in-toto attestations, and transparency log integration.

## Features

- Sigstore keyless signing (OIDC-based)
- in-toto attestation generation and verification
- SLSA provenance level tracking
- Rekor transparency log integration

## Usage

```rust
use bnpm_provenance::{Provenance, Signer};

let signer = Signer::from_oidc()?;
let prov = Provenance::new(&package, &signer).await?;
prov.upload_to_rekor().await?;
```
