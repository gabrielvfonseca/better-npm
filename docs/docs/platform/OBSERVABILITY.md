# Dash — Observability & Analytics
> **Status:** scoped · **Owner:** WS-PLATFORM · **Design refs:** DESIGN §19.6 · **Implements:** PLAT-M704 · **Conformance:** conformance/observability

**Audience:** platform

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §19.6. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements PLAT-M704 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Analytics, dependency-health dashboards, telemetry privacy model.

## Key points (from the design)

- Opt-in, policy-gated, anonymized telemetry powers dep-health, upgrade-lag, exposure-over-time dashboards (§19.6).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/observability`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
