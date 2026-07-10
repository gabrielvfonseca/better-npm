# Dash — Provenance & Transparency Log
> **Status:** scoped · **Owner:** WS-SECURITY · **Design refs:** DESIGN §13.2–§13.4 · **Implements:** REG-M305..307 · **Conformance:** conformance/provenance

**Audience:** platform, contributors

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §13.2–§13.4. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements REG-M305..307 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Sigstore/OIDC keyless signing, in-toto/SLSA attestations, witnessed transparency log, inclusion proofs.

## Key points (from the design)

- Every publish is signed (keyless OIDC) + attested (SLSA/in-toto) + appended to a witnessed log (§13.2).
- Clients verify a pinned CID offline via its inclusion proof; split-view is detectable (§13.2, §13.4).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/provenance`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
