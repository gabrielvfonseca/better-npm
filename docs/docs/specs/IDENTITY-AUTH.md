# Dash — Identity & Auth
> **Status:** scoped · **Owner:** WS-SECURITY · **Design refs:** DESIGN §14 · **Implements:** AI-M608, PLAT-M702 · **Conformance:** conformance/auth

**Audience:** platform, humans, agents

> **This is a scaffold.** The normative content lives in [`../../docs/DESIGN.md`](../../docs/DESIGN.md) at §14. This document *extracts and elaborates* that section; it must never contradict it. It is authored in full during the milestone that implements AI-M608, PLAT-M702 (see [`../../plan/BUILD-PLAN.md`](../../plan/BUILD-PLAN.md) and [`../../plan/TASK-BACKLOG.md`](../../plan/TASK-BACKLOG.md)).

## Purpose

did:dash identities, human/CI/agent auth, mandatory 2FA, fine-grained scoped grants, permission ACL.

## Key points (from the design)

- Humans: keyless OIDC + passkeys, mandatory 2FA to publish. CI: OIDC workload identity, no long-lived tokens (§14.1).
- Agents: scoped, time-boxed machine identities, audited per action (§14.1, §18.7).

## What this doc will contain when authored

- A precise, standalone specification of the above, with schemas/pseudocode where applicable.
- Machine-readable schema files shipped alongside (JSON Schema / CBOR schema) where relevant.
- The conformance slice that proves it (`conformance/auth`).
- Cross-links to neighboring specs and to the implementing tasks.

---

*Back to the [documentation index](../../README.md).*
