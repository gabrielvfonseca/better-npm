# Bnpm — Registry API
> **Status:** scoped · **Owner:** WS-REGISTRY · **Design refs:** DESIGN §12 · **Implements:** REG-M304..310 · **Conformance:** conformance/registry-api

**Audience:** platform, agents

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §12. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements REG-M304..310 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

REST/GraphQL/SDK surface for every registry capability; pagination, versioning, error model.

## Key points (from the design)

- Every capability is a documented, versioned API; the API is the product (§12.2).
- A machine surface at /.well-known/bnpm/ serves agents (§12.2, §18.6).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/registry-api`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
