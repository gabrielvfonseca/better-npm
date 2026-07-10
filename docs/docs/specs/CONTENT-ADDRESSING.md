# Dash — Content Addressing & Store
> **Status:** scoped · **Owner:** WS-CORE · **Design refs:** DESIGN §5 · **Implements:** CORE-M101..110 · **Conformance:** conformance/cid, conformance/store

**Audience:** platform, contributors

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §5. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements CORE-M101..110 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

CID construction (BLAKE3 multihash), canonicalization, capsule/file dedup, store layout.

## Key points (from the design)

- CID = `b3:` + base32(BLAKE3-256(canonical bytes)); BLAKE3 chosen for throughput + Merkle range proofs (§5.1).
- Two dedup levels: capsule and file; materialization via reflink→hardlink→copy (§5.3).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/cid, conformance/store`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
