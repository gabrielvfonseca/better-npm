# Dash — AI-Native Surface
> **Status:** scoped · **Owner:** WS-AI · **Design refs:** DESIGN §18 · **Implements:** AI-M601..609 · **Conformance:** conformance/ai

**Audience:** agents, platform

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §18. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements AI-M601..609 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

[ai] block, extracted API surface, verified examples, tool interfaces, agent endpoints.

## Key points (from the design)

- Agents reason instead of guess: intent taxonomy, declared side-effects, extracted types, verified examples (§18.1–§18.3).
- Endpoints select/explain/plan-upgrade/audit/compose return typed, cited results; autonomy bounded by the same capability model (§18.6–§18.7).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/ai`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
