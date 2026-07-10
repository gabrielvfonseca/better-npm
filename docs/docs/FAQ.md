# Dash — FAQ
> **Status:** planned · **Owner:** WS-DOC · **Design refs:** DESIGN §0.3 · **Implements:** — · **Conformance:** n/a

**Audience:** humans

> **This is a scaffold.** The normative content lives in [`../docs/DESIGN.md`](../docs/DESIGN.md) at §0.3. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements — (see [`../plan/BUILD-PLAN.md`](../plan/BUILD-PLAN.md) and [`../plan/TASK-BACKLOG.md`](../plan/TASK-BACKLOG.md)).

## Purpose

Objection-handling for adopters: why not just pnpm/Bun/JSR, what breaks, is it npm-compatible.

## Key points (from the design)

- Why not pnpm/Bun/JSR? — Dash reforms the *substrate* (identity, security, machine-legibility), not just the client (DESIGN §1).
- Is it npm-compatible? — Yes, incrementally and per-project via the bridge (§20).
- What breaks? — Lifecycle scripts, mutable versions, deep imports, hoisting (§21).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`n/a`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../README.md).*
