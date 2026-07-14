# `@dash/workers-signatures`

Signature verification worker.

## Overview

Verifies package cryptographic signatures using Node.js `crypto.verify()` with RSA-SHA512 and ECDSA-SHA512 support. Stores verification results for audit trail.

## Usage

```ts
import { handler } from "@dash/workers-signatures";

const result = await handler({
  name: "react",
  version: "18.2.0",
  signature: "base64...",
  algorithm: "rsa-sha512",
  publicKey: "-----BEGIN PUBLIC KEY-----...",
});
console.log(result.verified);
```
