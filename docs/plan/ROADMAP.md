# Bnpm — Roadmap

**Normative source:** [`../docs/DESIGN.md`](../docs/DESIGN.md) · **Companion:** [`BUILD-PLAN.md`](BUILD-PLAN.md)

This roadmap sequences Bnpm from an empty repository to general availability. It is deliberately **capability‑gated, not date‑gated**: a phase ends when its **exit gate** is provably met (a passing conformance suite, not a calendar). Rough calendar estimates assume a supervised fleet of ~8–15 engineering agents plus a small human steering group; treat them as order‑of‑magnitude, not commitments.

## Phase overview

| Phase | Name | Theme | Exit gate (provable) | Est. |
|---|---|---|---|---|
| **M0** | Spec‑lock | Freeze the contracts | All core schemas (manifest, lockfile, capsule, CID, DPR) ratified + JSON/CBOR schemas + a conformance skeleton that fails loudly | 2–3 wks |
| **M1** | Content core | The data plane in one machine | `bnpm` can hash, canonicalize, store, link, and verify capsules locally; bit‑reproducible; 100% CID round‑trip | 4–6 wks |
| **M2** | Resolve & install | Deterministic graphs | MVS resolver + lockfile + strict linking install an npm‑sourced graph offline, bit‑identical across machines | 5–7 wks |
| **M3** | Registry MVP | The control plane | Publish→transparency‑log→resolve→fetch works end‑to‑end over DPR with mandatory provenance | 6–8 wks |
| **M4** | Security & capabilities | No ambient authority | Hermetic builds, capability enforcement/gating, capability‑diff review, scanning, SBOM | 6–8 wks |
| **M5** | npm bridge & migration | Meet the ecosystem | Top‑1000 npm packages install & pass their own tests through Bnpm; dual‑publish works | 6–8 wks |
| **M6** | AI‑native surface | Agents as users | `select`/`explain`/`plan-upgrade`/`audit`/`compose` endpoints + `@better-npm/agent` + semantic index, all cited & auditable | 5–7 wks |
| **M7** | Platform & GA | Harden & scale | Federation, governance, reputation, observability; SLOs met; security audit passed; 1.0 spec frozen | 8–12 wks |

Phases overlap (see the DAG in [BUILD-PLAN](BUILD-PLAN.md)); the table lists the *critical path* ordering.

---

## M0 — Spec‑lock

**Goal:** make the contracts real and machine‑checkable before a line of product code exists, so every downstream agent builds against a fixed target.

Deliverables: ratified `DESIGN.md`; JSON Schema for `bnpm.toml`, `bnpm.lock`, the `[ai]` block, and `bnpm.policy.toml`; a CBOR schema for the capsule container and canonical manifest; the CID construction spec with test vectors; the DPR message schemas; a **conformance test skeleton** (all tests present, all currently failing/`skip`) that becomes the definition of "done" for every later phase; the extracted ✍️ spec docs listed in the [README index](../README.md).

**Exit gate:** a human steering review ratifies the schemas; `bnpm conformance --list` enumerates every required behavior; CI is green on "schemas validate against their own examples."

## M1 — Content core

**Goal:** the data plane, fully local. Everything content‑addressed, deterministic, verifiable — no network, no registry yet.

Deliverables: BLAKE3 CID library with test vectors; deterministic capsule packer/unpacker (`BNPMCAP1`); the global store (`~/.bnpm/store`) with capsule‑ and file‑level dedup; reflink→hardlink→copy materialization; `bnpm verify` for local content; a **reproducibility harness** proving two independent packs of the same tree yield the same CID.

**Exit gate:** pack→store→materialize→verify round‑trips for a 10k‑file corpus; byte‑identical capsules across Linux/macOS/Windows and across two machines; zero‑copy materialization confirmed on APFS/Btrfs.

## M2 — Resolve & install

**Goal:** turn a manifest into a deterministic, installed graph — still sourcing bytes locally (from a fixture registry / the npm bridge stub).

Deliverables: MVS resolver with singleton unification and the bounded PubGrub fallback; semver‑honesty checker (API‑surface diff) as a resolver input; `bnpm.lock` writer/reader with canonical ordering and self‑digest; strict non‑hoisted linking into `node_modules`; `bnpm install`, `bnpm add`, `bnpm up`, `bnpm why`, `bnpm install --offline`; workspace graph support.

**Exit gate:** the same manifest resolves to a byte‑identical `bnpm.lock` on every machine; a warm install does zero network I/O and is bit‑identical to a cold one; `bnpm why`/`--why` explains every selection; a 500‑package monorepo installs in one pass with full dedup.

## M3 — Registry MVP

**Goal:** the control plane end‑to‑end, with security‑critical properties on from day one.

Deliverables: name/version pointer service; append‑only **transparency log** with inclusion proofs and checkpoint signing; content service + CDN‑style FETCH; provenance/signing via Sigstore/OIDC (**mandatory**); DPR over HTTP/3 with HTTP/2 fallback; `bnpm publish` (single + dual‑publish stub); `bnpm verify` proving inclusion offline; a minimal web UI for browsing packages and their provenance.

**Exit gate:** publish a package from CI via OIDC (no long‑lived token), see it in the log with an inclusion proof, resolve+fetch+verify it from a clean machine offline‑after‑first‑fetch; a simulated split‑view registry is detected by the witness check.

## M4 — Security & capabilities

**Goal:** structurally remove ambient authority — the phase that defeats the supply‑chain‑worm class.

Deliverables: hermetic build sandbox (no net/fs/env/subprocess by default); declarative builders (`bnpm:tsc`, `bnpm:wasm`, `bnpm:node-gyp`) as content‑addressed capsules; runtime capability generation for Deno/Node permission models; **capability‑diff gating** in resolve/install; publish‑time + continuous scanning (malware heuristics, secret scan, typosquat/dependency‑confusion detection); SBOM (SPDX/CycloneDX) and license intelligence; `bnpm audit`, `bnpm policy check`, `bnpm.policy.toml` enforcement.

**Exit gate:** a red‑team package that adds a `net` request in a patch release is *blocked* at install pending review; an install‑time exploit attempt has nowhere to execute (no lifecycle scripts); SBOM matches the lock exactly; policy gates fail CI on a copyleft‑in‑proprietary violation.

## M5 — npm bridge & migration

**Goal:** meet the ecosystem where it is, at scale.

Deliverables: full compat bridge (npm tarball → canonical capsule → logged CID); `package.json` import and no‑manifest operation; `node_modules` compatibility mode passing the real toolchain; legacy‑script quarantine; dual‑publish to npm; the **compatibility corpus** harness (install + run each target package's own test suite through Bnpm).

**Exit gate:** the top‑1000 npm packages by downloads install through Bnpm and pass their own published tests at ≥ the rate they pass under npm; a real medium‑size app migrates via the 5‑step path with no code changes at step 1.

## M6 — AI‑native surface

**Goal:** make agents first‑class — grounded, cited, auditable.

Deliverables: extracted API‑surface index and version‑to‑version API diffs; verified‑examples runner; typed tool interfaces (`ai/tools.json`) + indexing; the semantic index (lexical + structural + vector); `/.well-known/bnpm/` endpoints and `@better-npm/agent` (`select`, `explain`, `plan-upgrade`, `audit`, `compose`, transactional `apply`); scoped agent identities.

**Exit gate:** an agent, given "add a retrying HTTP client, MIT/Apache, no native deps, SLSA≥3," selects, installs, and opens a PR whose lock `bnpm verify` passes and whose every claim links to the transparency log; `plan-upgrade` reports accurate API‑diff impact on a held‑out corpus.

## M7 — Platform & GA

**Goal:** operate at ecosystem scale and freeze 1.0.

Deliverables: federation + private/self‑hosted registry; governance (orgs/teams/roles/audit); reputation scoring; observability/analytics dashboards; performance hardening to SLOs; an external security audit; migration of internal dogfood monorepos; the frozen **Bnpm 1.0** spec and a compatibility promise.

**Exit gate:** published SLOs met under load (resolve p99, fetch throughput, publish latency); third‑party security audit closed with no criticals; ≥1 large external org running Bnpm in production; 1.0 spec ratified and versioned.

---

## Cross‑cutting tracks (run through every phase)

- **Documentation** keeps the ✍️ specs in lockstep with code (a merged behavior change that doesn't update its spec fails CI).
- **Reproducibility** — the determinism harness runs on every commit; a non‑reproducible artifact is a build break, not a warning.
- **Security** — threat‑model review at each phase gate; the red‑team suite grows monotonically.
- **DX** — every command ships with `--json`, `--why`, and a golden‑file CLI‑output test.

See [BUILD-PLAN.md](BUILD-PLAN.md) for the workstream/DAG detail and [TASK-BACKLOG.md](TASK-BACKLOG.md) for the executable task list.
