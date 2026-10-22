# Bnpm — Capsule Format (BNPMCAP1)
> **Status:** scoped · **Owner:** WS-CORE · **Design refs:** DESIGN §7 · **Implements:** CORE-M104..107 · **Conformance:** conformance/capsule

**Audience:** platform, contributors

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §7. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements CORE-M104..107 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

The BNPMCAP1 container, determinism guarantees, range-fetch layout, detached attestations.

## Key points (from the design)

- Header · manifest (CBOR) · sorted file table · zstd objects · detached attestations (§7.1).
- Deterministic (normalized metadata, sorted, fixed compression) → same tree, same CID; range-fetchable (§7.1).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/capsule`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
