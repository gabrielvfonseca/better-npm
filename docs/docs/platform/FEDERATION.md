# Bnpm — Federation
> **Status:** scoped · **Owner:** WS-PLATFORM · **Design refs:** DESIGN §12.3 · **Implements:** PLAT-M701 · **Conformance:** conformance/federation

**Audience:** platform

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §12.3. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements PLAT-M701 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Federated registries, upstream delegation, private+public boundary, npm bridge as upstream.

## Key points (from the design)

- A registry resolves names it doesn't own by delegating to upstreams, caching with provenance notes (§12.3).
- Enterprises get a single front door: private packages + local policy + federated reads (§12.3).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/federation`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
