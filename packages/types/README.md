# `@dash/types`

Shared TypeScript types and interfaces for the Dash ecosystem.

## Overview

Defines core types: `PackageId`, `PackageName`, `SemVer`, `PackageManifest`, `PackageLock`, `LockEntry`, `RegistryPackage`, `PackageVersionInfo`, `DistInfo`, `User`, `UserRole` enum, `ApiError`, `Result<T,E>`, `AsyncResult<T,E>`, `PaginatedResponse`, `PaginationParams`, `WorkerPayload`, `WorkerResult`, `EventType`.

## Usage

```ts
import type { PackageManifest, User, ApiError, Result } from "@dash/types";

const manifest: PackageManifest = { name: "react", version: "18.2.0" };
```
