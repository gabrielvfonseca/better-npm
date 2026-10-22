# Bnpm — Registry Services
> **Status:** scoped · **Owner:** WS-REGISTRY · **Design refs:** DESIGN §12.1 · **Implements:** REG-* · **Conformance:** conformance/registry

**Audience:** platform

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §12.1. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements REG-* (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Service decomposition: name/version, transparency log, content, provenance, index, policy, reputation, analytics.

## Key points (from the design)

- The only stateful trust root is the small name→version→CID service; everything else is derived or pure content (§12.1).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/registry`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
