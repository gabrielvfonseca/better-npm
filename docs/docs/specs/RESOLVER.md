# Bnpm — Resolver (MVS + PubGrub)
> **Status:** scoped · **Owner:** WS-RESOLVE · **Design refs:** DESIGN §9, App. C · **Implements:** RES-M203..207 · **Conformance:** conformance/resolver

**Audience:** platform, contributors

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §9, App. C. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements RES-M203..207 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

MVS algorithm, singleton unification, PubGrub fallback, semver-honesty verification, complexity.

## Key points (from the design)

- MVS = lowest version satisfying all requirements; deterministic, no SAT, minimal explainable diffs (§9.1).
- SAT/PubGrub only for real conflicts; semver-honesty (API-diff) makes MVS trustworthy (§9.1).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/resolver`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
