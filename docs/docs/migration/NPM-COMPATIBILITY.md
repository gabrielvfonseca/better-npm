# Bnpm — npm Compatibility
> **Status:** scoped · **Owner:** WS-BRIDGE · **Design refs:** DESIGN §20 · **Implements:** BRIDGE-M501..505 · **Conformance:** conformance/bridge, corpus

**Audience:** humans

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §20. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements BRIDGE-M501..505 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

The compat bridge, package.json import, node_modules mode, dual-publish, legacy-script quarantine.

## Key points (from the design)

- npm tarballs are canonicalized into capsules with logged provenance; npm becomes a federated upstream (§20.1).
- node_modules compat mode runs the whole existing toolchain; legacy scripts are quarantined, never ambient (§20.1, §20.3).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/bridge, corpus`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
