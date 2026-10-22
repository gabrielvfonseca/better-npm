# `@better-npm/apps-auth`

Authentication service — Hono-based service for user auth.

## Overview

Hono-based auth service handling login, registration, token refresh, logout, session verification, and session lookup using JWT tokens and PBKDF2 password hashing from `@better-npm/auth`.

## Endpoints

- `POST /auth/login` — Authenticate user
- `POST /auth/register` — Create account
- `POST /auth/refresh` — Refresh access token
- `POST /auth/logout` — Invalidate session
- `GET /auth/verify` — Verify token validity
- `GET /auth/session` — Get current session

## Usage

```ts
import { createApp } from "@better-npm/apps-auth";

const app = createApp({ port: 3001 });
```
