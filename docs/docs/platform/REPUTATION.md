# Dash — Reputation Scoring
> **Status:** scoped · **Owner:** WS-PLATFORM · **Design refs:** DESIGN §19.4 · **Implements:** PLAT-M703 · **Conformance:** conformance/reputation

**Audience:** platform

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §19.4. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements PLAT-M703 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Multi-signal scoring model, Sybil/manipulation resistance, inputs and weights.

## Key points (from the design)

- Weights *verified* signals (provenance, verified production use, semver honesty) over spoofable ones (raw downloads/stars) (§19.4).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/reputation`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
