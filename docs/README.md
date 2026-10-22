# Bnpm — Documentation Set

**Bnpm** is a content‑addressed, cryptographically verifiable, ESM‑first package ecosystem for JavaScript and TypeScript — a clean‑sheet successor to npm designed for reproducible builds, capability‑based security, and AI agents as first‑class users.

This repository holds the complete documentation set: the canonical design, the layered specifications extracted from it, the platform docs, the migration story, and — the focus of this bundle — the **agentic implementation and build plan** that a fleet of AI engineering agents (supervised by humans) can execute end‑to‑end.

> New here? Read in this order: **[DESIGN](docs/DESIGN.md)** → **[plan/ROADMAP](plan/ROADMAP.md)** → **[plan/BUILD-PLAN](plan/BUILD-PLAN.md)** → **[plan/AGENT-PLAYBOOK](plan/AGENT-PLAYBOOK.md)** → **[plan/TASK-BACKLOG](plan/TASK-BACKLOG.md)**.

---

## The comprehensive list of documents

Legend — **Status:** ✅ authored in this bundle · ✍️ scoped, extract from `DESIGN.md` §refs · 🔭 planned. **Audience:** 👤 human devs · 🏛️ platform/registry ops · 🤖 agents/automation · 🧭 contributors.

### 0. Canonical reference

| # | Document | Purpose | Status | Audience |
|---|---|---|---|---|
| 0.1 | [`docs/DESIGN.md`](docs/DESIGN.md) | The single source of truth: vision, principles, full architecture, protocol, registry, security, AI‑native layer, migration, appendices. Everything else is a projection of this. | ✅ | 👤🏛️🤖🧭 |
| 0.2 | [`docs/GLOSSARY.md`](docs/GLOSSARY.md) | Canonical definitions of every term (capsule, CID, MVS, transparency log, capability, DPR, etc.). One word, one meaning, everywhere. | ✍️ | 👤🤖 |
| 0.3 | [`docs/FAQ.md`](docs/FAQ.md) | "Why not just pnpm/Bun/JSR?", "Is this npm‑compatible?", "What breaks?" — objection handling for adopters. | 🔭 | 👤 |

### 1. Foundations

| # | Document | Purpose | Status | Audience |
|---|---|---|---|---|
| 1.1 | [`docs/VISION.md`](docs/VISION.md) | The thesis and the three first‑class constituencies (humans, CI, agents). Extract of DESIGN §2. | ✍️ | 👤🏛️ |
| 1.2 | [`docs/PRINCIPLES.md`](docs/PRINCIPLES.md) | The ten guiding principles and how each maps to a concrete mechanism. Extract of DESIGN §3. | ✍️ | 👤🧭 |
| 1.3 | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Data‑plane / control‑plane split, component map, trust boundaries, diagrams. Extract of DESIGN §4, §22. | ✍️ | 🏛️🧭 |

### 2. Core specifications (the data plane)

| # | Document | Purpose | Status | Audience |
|---|---|---|---|---|
| 2.1 | [`docs/specs/CONTENT-ADDRESSING.md`](docs/specs/CONTENT-ADDRESSING.md) | CID construction (BLAKE3 multihash), canonicalization rules, capsule‑ and file‑level dedup, the store layout. DESIGN §5. | ✍️ | 🏛️🧭 |
| 2.2 | [`docs/specs/CAPSULE-FORMAT.md`](docs/specs/CAPSULE-FORMAT.md) | The `BNPMCAP1` container: header, file table, objects, detached attestations; determinism guarantees; range‑fetch layout. DESIGN §7. | ✍️ | 🏛️🧭 |
| 2.3 | [`docs/specs/MANIFEST.md`](docs/specs/MANIFEST.md) | `bnpm.toml` schema — identity, exports, deps, capabilities, build, `[ai]`, workspace. Field‑by‑field, with validation rules. DESIGN §6. | ✍️ | 👤🤖 |
| 2.4 | [`docs/specs/LOCKFILE.md`](docs/specs/LOCKFILE.md) | `bnpm.lock` grammar, determinism, content‑pinning, inclusion proofs, merge semantics. DESIGN §8, App. B. | ✍️ | 👤🤖 |
| 2.5 | [`docs/specs/RESOLVER.md`](docs/specs/RESOLVER.md) | MVS algorithm, singleton unification, PubGrub fallback, semver‑honesty verification, complexity. DESIGN §9, App. C. | ✍️ | 🏛️🧭 |
| 2.6 | [`docs/specs/STORAGE-AND-CACHE.md`](docs/specs/STORAGE-AND-CACHE.md) | Global store, L0–L4 cache hierarchy, reflink/hardlink materialization, offline mode, vendoring. DESIGN §5.3, §10. | ✍️ | 🏛️🧭 |

### 3. Protocol & registry (the control plane)

| # | Document | Purpose | Status | Audience |
|---|---|---|---|---|
| 3.1 | [`docs/specs/PROTOCOL-DPR.md`](docs/specs/PROTOCOL-DPR.md) | DPR wire protocol: `RESOLVE`/`FETCH`/`PUBLISH`, HTTP/3 transport, delta transfer, range proofs. DESIGN §11, App. D. | ✍️ | 🏛️🧭 |
| 3.2 | [`docs/specs/REGISTRY-API.md`](docs/specs/REGISTRY-API.md) | REST/GraphQL/SDK surface for every registry capability; pagination, versioning, error model. DESIGN §12. | ✍️ | 🏛️🤖 |
| 3.3 | [`docs/platform/REGISTRY-SERVICES.md`](docs/platform/REGISTRY-SERVICES.md) | Service decomposition: name/version, transparency log, content, provenance, index, policy, reputation, analytics. DESIGN §12.1. | ✍️ | 🏛️ |
| 3.4 | [`docs/platform/FEDERATION.md`](docs/platform/FEDERATION.md) | Federated registries, upstream delegation, private + public boundary, the npm bridge as an upstream. DESIGN §12.3. | ✍️ | 🏛️ |
| 3.5 | [`docs/platform/GOVERNANCE-POLICY.md`](docs/platform/GOVERNANCE-POLICY.md) | Orgs, teams, roles, `bnpm.policy.toml`, policy‑as‑code enforcement points. DESIGN §13.5, §14, §19.5. | ✍️ | 🏛️👤 |
| 3.6 | [`docs/platform/REPUTATION.md`](docs/platform/REPUTATION.md) | Multi‑signal scoring model, Sybil/manipulation resistance, inputs and weights. DESIGN §19.4. | ✍️ | 🏛️ |
| 3.7 | [`docs/platform/OBSERVABILITY.md`](docs/platform/OBSERVABILITY.md) | Analytics, dependency‑health dashboards, telemetry privacy model. DESIGN §19.6. | ✍️ | 🏛️ |

### 4. Security & identity

| # | Document | Purpose | Status | Audience |
|---|---|---|---|---|
| 4.1 | [`docs/specs/SECURITY-MODEL.md`](docs/specs/SECURITY-MODEL.md) | The five defense layers, capability model, hermetic builds, capability‑diff gating, threat model & attack walkthroughs (incl. worm class). DESIGN §13. | ✍️ | 🏛️🧭 |
| 4.2 | [`docs/specs/PROVENANCE.md`](docs/specs/PROVENANCE.md) | Sigstore/OIDC keyless signing, in‑toto/SLSA attestations, the witnessed transparency log, inclusion proofs. DESIGN §13.2–§13.4. | ✍️ | 🏛️🧭 |
| 4.3 | [`docs/specs/IDENTITY-AUTH.md`](docs/specs/IDENTITY-AUTH.md) | `did:bnpm` identities, human/CI/agent auth, mandatory 2FA, fine‑grained scoped grants, permission ACL. DESIGN §14. | ✍️ | 🏛️👤🤖 |
| 4.4 | [`docs/specs/SBOM-LICENSE.md`](docs/specs/SBOM-LICENSE.md) | SPDX/CycloneDX generation from the lock, SPDX expression resolution, license‑change detection. DESIGN §13.6. | ✍️ | 🏛️👤 |

### 5. Clients, SDKs, and the AI‑native surface

| # | Document | Purpose | Status | Audience |
|---|---|---|---|---|
| 5.1 | [`docs/specs/CLI.md`](docs/specs/CLI.md) | `bnpm` command surface, flags, exit codes, `--json` output contracts, zero‑config behavior. DESIGN §16. | ✍️ | 👤🤖 |
| 5.2 | [`docs/specs/SDK.md`](docs/specs/SDK.md) | `@better-npm/core`, `@better-npm/registry`, `@better-npm/sbom`, `@better-npm/agent` — package boundaries and public APIs. DESIGN §17. | ✍️ | 👤🤖 |
| 5.3 | [`docs/specs/AI-NATIVE.md`](docs/specs/AI-NATIVE.md) | `[ai]` block, extracted API surface, verified examples, tool interfaces, `/.well-known/bnpm/` agent endpoints (`select`/`explain`/`plan-upgrade`/`audit`/`compose`). DESIGN §18. | ✍️ | 🤖🏛️ |
| 5.4 | [`docs/specs/SEMANTIC-SEARCH.md`](docs/specs/SEMANTIC-SEARCH.md) | Lexical + structural + vector index design, ranking, the query grammar shared by humans and agents. DESIGN §19.2. | ✍️ | 🏛️🤖 |

### 6. Extensibility & migration

| # | Document | Purpose | Status | Audience |
|---|---|---|---|---|
| 6.1 | [`docs/specs/EXTENSIBILITY.md`](docs/specs/EXTENSIBILITY.md) | Registry plugins, builder capsules, webhooks/event stream, the public API as the extension surface. DESIGN §19.7. | ✍️ | 🏛️🧭 |
| 6.2 | [`docs/migration/NPM-COMPATIBILITY.md`](docs/migration/NPM-COMPATIBILITY.md) | The compat bridge, `package.json` import, `node_modules` mode, dual‑publish, legacy‑script quarantine. DESIGN §20. | ✍️ | 👤 |
| 6.3 | [`docs/migration/MIGRATION-GUIDE.md`](docs/migration/MIGRATION-GUIDE.md) | The five‑step staged adoption path with copy‑paste commands and per‑step wins. DESIGN §20.4. | 🔭 | 👤 |

### 7. The agentic implementation & build plan  ⭐

| # | Document | Purpose | Status | Audience |
|---|---|---|---|---|
| 7.1 | [`plan/ROADMAP.md`](plan/ROADMAP.md) | Phased milestones (M0–M7) from spec‑lock to GA, with the exit gate for each phase. | ✅ | 👤🏛️🤖 |
| 7.2 | [`plan/BUILD-PLAN.md`](plan/BUILD-PLAN.md) | **The core plan.** Workstreams, module build order, the dependency DAG, per‑component acceptance criteria, and how work is parallelized across agents. | ✅ | 🤖🧭 |
| 7.3 | [`plan/AGENT-PLAYBOOK.md`](plan/AGENT-PLAYBOOK.md) | How agents actually execute: roles, the task lifecycle, coordination protocol, guardrails, Definition of Done, escalation rules. | ✅ | 🤖🧭 |
| 7.4 | [`plan/TASK-BACKLOG.md`](plan/TASK-BACKLOG.md) | The granular, ID'd, dependency‑ordered task list agents pull from — epics → tasks with acceptance criteria and agent role. | ✅ | 🤖 |
| 7.5 | [`plan/TESTING-STRATEGY.md`](plan/TESTING-STRATEGY.md) | Determinism/reproducibility harness, conformance suite, fuzzing, security red‑team, the npm‑ecosystem compatibility corpus. | ✅ | 🤖🧭 |
| 7.6 | [`plan/TECH-STACK.md`](plan/TECH-STACK.md) | Language/tooling choices per component (Rust core, TS SDKs, service stack) with rationale and the "boring by default" rule. | ✅ | 🧭🏛️ |

### 8. Contributing & process

| # | Document | Purpose | Status | Audience |
|---|---|---|---|---|
| 8.1 | [`contributing/CONTRIBUTING.md`](contributing/CONTRIBUTING.md) | Repo layout, dev environment, commit/PR conventions, how humans and agents co‑develop. | 🔭 | 🧭🤖 |
| 8.2 | [`contributing/RFC-PROCESS.md`](contributing/RFC-PROCESS.md) | How the spec itself evolves: RFC lifecycle, who ratifies, how `DESIGN.md` changes. | 🔭 | 🧭 |
| 8.3 | [`contributing/CODE-OF-CONDUCT.md`](contributing/CODE-OF-CONDUCT.md) | Community standards. | 🔭 | 🧭 |

---

## Repository layout

```
bnpm-docs/
├── README.md                     ← you are here (the comprehensive doc list)
├── docs/
│   ├── DESIGN.md                 ← canonical design (source of truth)
│   ├── VISION.md · PRINCIPLES.md · ARCHITECTURE.md · GLOSSARY.md · FAQ.md
│   ├── specs/                    ← layered specifications (data + control plane)
│   ├── platform/                 ← registry-as-a-platform docs
│   └── migration/                ← npm compatibility & adoption
├── plan/                         ← ⭐ the agentic implementation & build plan
│   ├── ROADMAP.md
│   ├── BUILD-PLAN.md
│   ├── AGENT-PLAYBOOK.md
│   ├── TASK-BACKLOG.md
│   ├── TESTING-STRATEGY.md
│   └── TECH-STACK.md
└── contributing/                 ← process & community
```

## Authoring conventions (so humans and agents stay in sync)

- **`DESIGN.md` is normative.** Every ✍️ spec doc is an *extraction and elaboration* of the referenced DESIGN section — it may add detail but must never contradict it. A change in behavior lands in `DESIGN.md` first (via the RFC process), then flows down.
- **Every spec doc carries a header block:** `status`, `owner`, `design-refs`, `implements` (the epic/task IDs from the backlog), and `conformance` (the test suite that proves it).
- **One term, one meaning.** New terms are defined in `GLOSSARY.md` before use.
- **Machine‑readable co‑equal.** Any schema shown in a spec also ships as a JSON Schema / CBOR schema next to it, so agents validate rather than parse prose.

## Status of this bundle

The **plan suite (§7)** and this index are authored in full — that is the deliverable requested. The design (§0.1) is complete and normative. The ✍️ specification, platform, and migration docs are fully *scoped* here (each row states its purpose and the exact `DESIGN.md` sections it extracts) and are the first content the documentation workstream produces in **M0** (see [BUILD-PLAN](plan/BUILD-PLAN.md) → WS‑DOC). Ask to have any single ✍️/🔭 row expanded into a complete standalone file.
