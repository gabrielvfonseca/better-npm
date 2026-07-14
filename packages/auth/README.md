# `@dash/auth`

Cryptographic authentication utilities: JWT, PBKDF2, and token parsing.

## Overview

Pure crypto-based auth utilities including JWT creation/verification (HMAC-SHA256), password hashing (PBKDF2), and Bearer token extraction.

## Usage

```ts
import { createToken, verifyToken, hashPassword, verifyPassword } from "@dash/auth";

const token = await createToken({ userId: "123", role: "admin" }, "secret");
const decoded = await verifyToken(token, "secret");

const hash = await hashPassword("my-password");
const match = await verifyPassword("my-password", hash);
```
