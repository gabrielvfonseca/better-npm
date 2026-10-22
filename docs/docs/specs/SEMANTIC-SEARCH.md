# Bnpm — Semantic Search
> **Status:** scoped · **Owner:** WS-AI · **Design refs:** DESIGN §19.2 · **Implements:** AI-M605 · **Conformance:** conformance/search

**Audience:** platform, agents

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §19.2. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements AI-M605 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Lexical + structural + vector index design, ranking, the query grammar shared by humans and agents.

## Key points (from the design)

- Three indexes over one corpus (lexical, structural/API-shape, semantic/vector) with explainable ranking (§19.2).
- Same endpoint serves `bnpm search` and the agent `select` call (§19.2).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/search`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
