# Bnpm — Lockfile (bnpm.lock)
> **Status:** scoped · **Owner:** WS-RESOLVE · **Design refs:** DESIGN §8, App. B · **Implements:** RES-M208 · **Conformance:** conformance/lock

**Audience:** humans, agents

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §8, App. B. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements RES-M208 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

bnpm.lock grammar, determinism, content-pinning, inclusion proofs, merge semantics.

## Key points (from the design)

- Text, sorted, merge-friendly, content-pinned to CIDs, self-verifying via a top-level digest + inclusion proofs (§8).
- Manifest states intent; lock states facts. Agents edit intent and regenerate facts (§8).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/lock`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
