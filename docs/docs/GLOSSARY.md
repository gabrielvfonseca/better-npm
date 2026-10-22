# Bnpm — Glossary
> **Status:** scoped · **Owner:** WS-DOC · **Design refs:** DESIGN §0.3 · **Implements:** DOC-M001 · **Conformance:** n/a

**Audience:** humans, agents

> **This is a scaffold.** The normative content lives in [`../docs/DESIGN.md`](../docs/DESIGN.md) at §0.3. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements DOC-M001 (see [`../plan/BUILD-PLAN.md`](../plan/BUILD-PLAN.md) and [`../plan/TASK-BACKLOG.md`](../plan/TASK-BACKLOG.md)).

## Purpose

Canonical definitions of every Bnpm term. One word, one meaning, everywhere.

## Key points (from the design)

- **Capsule** — immutable, content-addressed archive of one package version (DESIGN §5.1).
- **CID** — `b3:` BLAKE3 content identifier; the artifact's true identity (§5.1).
- **Control plane / data plane** — mutable authoritative pointers vs. pure cacheable content (§4).
- **MVS** — Minimal Version Selection, the default resolver (§9.1).
- **Transparency log** — append-only, witnessed record of every publish (§13.2).
- **Capability** — a declared, policy-granted privilege (`net`, `fs`, `env`, `subprocess`) (§13.3).
- **DPR** — Bnpm Package Registry protocol (`RESOLVE`/`FETCH`/`PUBLISH`) (§11).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`n/a`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../README.md).*
