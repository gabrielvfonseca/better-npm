# Bnpm — SBOM & License Intelligence
> **Status:** scoped · **Owner:** WS-SECURITY · **Design refs:** DESIGN §13.6 · **Implements:** SEC-M407 · **Conformance:** conformance/sbom

**Audience:** platform, humans

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §13.6. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements SEC-M407 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

SPDX/CycloneDX generation from the lock, SPDX expression resolution, license-change detection.

## Key points (from the design)

- SBOM is exact (generated from the content-pinned lock), not reconstructed (§13.6).
- License graph resolved as SPDX expressions; permissive→copyleft relicense is a policy event (§13.6).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/sbom`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
