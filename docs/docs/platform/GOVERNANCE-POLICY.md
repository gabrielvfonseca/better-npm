# Dash — Governance & Policy
> **Status:** scoped · **Owner:** WS-PLATFORM · **Design refs:** DESIGN §13.5, §14, §19.5 · **Implements:** PLAT-M702, SEC-M408 · **Conformance:** conformance/policy

**Audience:** platform, humans

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §13.5, §14, §19.5. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements PLAT-M702, SEC-M408 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

Orgs, teams, roles, dash.policy.toml, policy-as-code enforcement points.

## Key points (from the design)

- Policy-as-code: licenses, min SLSA, required provenance, capability allowlists, version cooldowns (§13.5).
- Capability ACL over who/what/scope; every grant auditable and revocable (§14.2).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/policy`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
