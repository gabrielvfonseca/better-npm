# `@dash/workers`

Background worker collection for Dash registry operations.

## Overview

Workers are background job processors that handle asynchronous registry operations. Each worker is a separate package under `apps/workers/`.

## Workers

| Package | Purpose |
|---------|---------|
| `analytics` | Processes and enriches analytics events in batches |
| `cleanup` | Removes stale cache entries, temp files, old package versions, logs, and orphaned metadata |
| `downloads` | Downloads package tarballs with SHA-512 integrity verification |
| `indexing` | Indexes/reindexes/removes package documents in the search service |
| `metadata` | Enriches package metadata: GitHub parsing, SPDX lookup, keyword normalization, dependency graph |
| `scanning` | Scans packages for security vulnerabilities (eval, XSS, secrets, injection, prototype pollution) |
| `signatures` | Verifies package signatures via RSA-SHA512 and ECDSA-SHA512 |
