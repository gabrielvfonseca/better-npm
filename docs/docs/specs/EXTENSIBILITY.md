# Dash — Extensibility
> **Status:** scoped · **Owner:** WS-PLATFORM · **Design refs:** DESIGN §19.7 · **Implements:** PLAT-M705 · **Conformance:** conformance/plugins

**Audience:** platform, contributors

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §19.7. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements PLAT-M705 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Registry plugins, builder capsules, webhooks/event stream, the public API as extension surface.

## Key points (from the design)

- Registry plugins and builder capsules run sandboxed with declared capabilities — same model as packages (§19.7).
- Webhooks + typed event stream + a stable public API are the universal extension surface (§19.7).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/plugins`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
