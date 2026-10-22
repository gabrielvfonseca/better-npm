# Bnpm — Architecture
> **Status:** scoped · **Owner:** WS-DOC · **Design refs:** DESIGN §4, §22 · **Implements:** — · **Conformance:** conformance/architecture

**Audience:** platform, contributors

> **This is a scaffold.** The normative content lives in [`../docs/DESIGN.md`](../docs/DESIGN.md) at §4, §22. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements — (see [`../plan/BUILD-PLAN.md`](../plan/BUILD-PLAN.md) and [`../plan/TASK-BACKLOG.md`](../plan/TASK-BACKLOG.md)).

## Purpose

Data-plane / control-plane split, component map, trust boundaries, diagrams.

## Key points (from the design)

- Data plane = pure content, any mirror, verified locally; control plane = small authoritative pointers + proofs (§4).
- The split is also the build strategy: prove the data plane offline first (BUILD-PLAN §1).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/architecture`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../README.md).*
