# `@better-npm/types`

Shared TypeScript types and interfaces for the Bnpm ecosystem.

## Overview

Defines core types: `PackageId`, `PackageName`, `SemVer`, `PackageManifest`, `PackageLock`, `LockEntry`, `RegistryPackage`, `PackageVersionInfo`, `DistInfo`, `User`, `UserRole` enum, `ApiError`, `Result<T,E>`, `AsyncResult<T,E>`, `PaginatedResponse`, `PaginationParams`, `WorkerPayload`, `WorkerResult`, `EventType`.

## Usage

```ts
import type { PackageManifest, User, ApiError, Result } from "@better-npm/types";

const manifest: PackageManifest = { name: "react", version: "18.2.0" };
```
