# Dash — Manifest (dash.toml)
> **Status:** scoped · **Owner:** WS-RESOLVE · **Design refs:** DESIGN §6 · **Implements:** RES-M201..202 · **Conformance:** conformance/manifest

**Audience:** humans, agents

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §6. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements RES-M201..202 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Field-by-field dash.toml schema: identity, exports, deps, capabilities, build, [ai], workspace.

## Key points (from the design)

- Capabilities declared in-manifest, granted by consumer policy; typed condition-based `exports` are the only entry points (§6).
- The `[ai]` block is validated on publish and indexed; `package.json` is a first-class import (§6, §20).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/manifest`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
