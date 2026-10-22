# Bnpm — DPR Wire Protocol
> **Status:** scoped · **Owner:** WS-REGISTRY · **Design refs:** DESIGN §11, App. D · **Implements:** REG-M301..303 · **Conformance:** conformance/dpr

**Audience:** platform, contributors

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §11, App. D. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements REG-M301..303 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

DPR verbs, HTTP/3 transport, delta transfer, BLAKE3 range proofs, fallback.

## Key points (from the design)

- Three verbs: RESOLVE (control), FETCH (data, any mirror), PUBLISH (control) (§11.2).
- HTTP/3 with HTTP/2→HTTPS fallback; delta-aware transfer sends only changed file objects (§11.1, §11.3).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/dpr`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
