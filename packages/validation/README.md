# `@better-npm/validation`

Validation utilities wrapping Zod with Result types.

## Overview

Wraps `@better-npm/schemas` Zod schemas with convenient validation helpers: `validate()` (safe parse returning `Result`), `assert()` (throws `ValidationError` on invalid), and `isValid()` (boolean check).

## Usage

```ts
import { validate, assert, isValid } from "@better-npm/validation";
import { semVerSchema } from "@better-npm/schemas";

const result = validate(semVerSchema, "1.0.0"); // Result<SemVer, ValidationError>
assert(semVerSchema, "1.0.0"); // throws if invalid
const valid = isValid(semVerSchema, "not-version"); // false
```
