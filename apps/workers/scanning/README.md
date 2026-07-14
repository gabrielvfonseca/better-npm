# `@dash/workers-scanning`

Security scanning worker.

## Overview

Scans package files for security vulnerabilities: `eval()` usage, `innerHTML` XSS, hardcoded secrets/API keys, command injection patterns, prototype pollution, and dependency analysis (wildcard version detection).

## Usage

```ts
import { handler } from "@dash/workers-scanning";

const result = await handler({ name: "react", version: "18.2.0", files: { "index.js": "console.log('hello')" } });
console.log(result.vulnerabilities);
```
