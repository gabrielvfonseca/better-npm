# Bnpm — CLI (bnpm)
> **Status:** scoped · **Owner:** WS-RESOLVE · **Design refs:** DESIGN §16 · **Implements:** RES-M210 · **Conformance:** conformance/cli

**Audience:** humans, agents

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §16. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements RES-M210 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

bnpm command surface, flags, exit codes, --json contracts, zero-config behavior.

## Key points (from the design)

- Zero-config, fast, explainable (`--why` everywhere), honest (`--json` everywhere) (§16).
- A bare `bnpm install` in an npm project just works via the bridge (§16, §20).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/cli`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
