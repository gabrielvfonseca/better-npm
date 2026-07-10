# Dash — Security Model
> **Status:** scoped · **Owner:** WS-SECURITY · **Design refs:** DESIGN §13 · **Implements:** SEC-M401..409 · **Conformance:** conformance/security, red-team

**Audience:** platform, contributors

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §13. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements SEC-M401..409 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

The five defense layers, capability model, hermetic builds, capability-diff gating, threat model.

## Key points (from the design)

- Layers: immutable identity · mandatory logged provenance · no ambient authority · continuous verification · policy-as-code (§13).
- The attack isn't the malicious code, it's the ambient privilege it inherits — remove it and worm-class attacks have nowhere to land (§13.3).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/security, red-team`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
