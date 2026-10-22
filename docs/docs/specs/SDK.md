# Bnpm — SDKs
> **Status:** scoped · **Owner:** WS-AI · **Design refs:** DESIGN §17 · **Implements:** AI-M607 · **Conformance:** conformance/sdk

**Audience:** humans, agents

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §17. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements AI-M607 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

@better-npm/core, @better-npm/registry, @better-npm/sbom, @better-npm/agent — package boundaries and public APIs.

## Key points (from the design)

- Everything the CLI does is a typed library; the CLI is built on it (dogfooded parity) (§17).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/sdk`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
