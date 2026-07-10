# Dash — Storage & Cache
> **Status:** scoped · **Owner:** WS-CORE · **Design refs:** DESIGN §5.3, §10 · **Implements:** CORE-M108..110 · **Conformance:** conformance/cache

**Audience:** platform, contributors

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §5.3, §10. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements CORE-M108..110 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Global store, L0–L4 cache hierarchy, materialization, offline mode, vendoring.

## Key points (from the design)

- Same content hash addresses every cache layer → verified promotion, trustless org sharing (§10.1).
- Store is source of truth: warm install = zero network, bit-identical to online (§10.2).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/cache`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
