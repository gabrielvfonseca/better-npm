# Dash — Vision
> **Status:** scoped · **Owner:** WS-DOC · **Design refs:** DESIGN §2 · **Implements:** — · **Conformance:** n/a

**Audience:** humans, platform

> **This is a scaffold.** The normative content lives in [`../docs/DESIGN.md`](../docs/DESIGN.md) at §2. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements — (see [`../plan/BUILD-PLAN.md`](../plan/BUILD-PLAN.md) and [`../plan/TASK-BACKLOG.md`](../plan/TASK-BACKLOG.md)).

## Purpose

The thesis and the three first-class constituencies (humans, CI, agents).

## Key points (from the design)

- A build is a pure function of a lockfile; code cannot act without a declared capability; every package is machine-legible (§2).
- Humans get zero-config offline speed; CI gets reproducibility + attestations; agents get structured selection/audit/upgrade (§2).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`n/a`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../README.md).*
