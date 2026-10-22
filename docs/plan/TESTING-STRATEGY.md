# Bnpm — Testing & Verification Strategy

In Bnpm, tests are not a safety net bolted on after the fact — **the conformance suite *is* the specification's executable form, and "done" is defined as "the relevant tests are green"** ([AGENT-PLAYBOOK §4](AGENT-PLAYBOOK.md)). This document defines the test layers, what each proves, and how they gate the agentic build.

The guiding stance mirrors the product's own philosophy (*verify, don't trust*): acceptance is evidence‑based and independently re‑run, never asserted.

## 1. The test pyramid, Bnpm‑specific

| Layer | Proves | Runs | Gates |
|---|---|---|---|
| **Unit** | A function does what it says | per‑crate/package | every PR |
| **Property/fuzz** | No input breaks an invariant (esp. the packer/encoder) | randomized | every PR (nightly deep) |
| **Conformance** | Behavior matches the ratified spec + test vectors | cross‑impl | every PR |
| **Reproducibility** | Byte‑identical outputs across machines/OSes | 2+ OS matrix | every PR |
| **Integration/e2e** | Components compose (resolve→install, publish→verify) | scenario | affected PRs + nightly |
| **Compatibility corpus** | Real npm packages work through Bnpm | top‑N harness | nightly (M5+) |
| **Security red‑team** | Known attack classes are blocked | adversarial corpus | every PR (grows monotonically) |
| **Load/SLO** | Meets latency/throughput targets | perf lab | pre‑gate (M3, M7) |

## 2. The four verification pillars unique to Bnpm

### 2.1 Determinism / reproducibility harness
The single most important suite. It packs, stores, and materializes a large fixture corpus on **at least two OSes** (Linux + macOS, Windows in matrix) and on two independent machines, then asserts **byte‑identical CIDs, capsules, lockfiles, and materialized trees**. Any nondeterminism is a *build break*, not a warning. Enforced upstream by a lint in `crates/` that bans wall‑clock, RNG, locale‑sensitive ops, and unordered‑map iteration in core paths. This suite exists from M1 and every core component must pass it before merge — it de‑risks the project's central thesis first (BUILD‑PLAN §8).

### 2.2 Cross‑implementation conformance vectors
Because Bnpm intends multiple implementations over time (and because agents shouldn't grade their own homework), correctness is pinned to **published test vectors**: CID vectors, canonical‑encoding vectors, capsule fixtures, resolver input→lock oracles, DPR message fixtures. A second, independent implementation (even a minimal reference one) must reproduce them. This turns "is this correct?" into "does it match the vector?" — a mechanical check the Verifier role applies.

### 2.3 Resolver oracle testing
The resolver is validated against a **reference oracle** on a large labeled graph corpus: MVS selections, singleton unification outcomes, and PubGrub conflict cases with their expected human/agent‑readable explanations. Semver‑honesty is tested on a corpus of real version pairs labeled breaking/non‑breaking (mined from the npm history via the bridge), measuring precision/recall of the API‑diff checker. This is where SPIKE‑03 (accuracy on dynamic TS) feeds in.

### 2.4 Security red‑team corpus
An adversarial suite that grows every phase and never shrinks. It plants, and asserts Bnpm blocks or neutralizes:
- **Worm‑class** (Shai‑Hulud style): a package that, on a new version, tries to execute at install → must have **no execution point** (no lifecycle scripts).
- **Capability escalation:** a patch release adding `net`/`subprocess` → **blocked pending review** with a legible diff.
- **Dependency confusion / typosquat:** name‑similar + ownership + install‑pattern heuristics → **flagged/quarantined**.
- **Secret exfiltration:** a build reaching for `env`/network → **denied** by the hermetic sandbox.
- **Split‑view registry:** an origin serving different histories → **detected** by witness cosigning.
- **Tampered bytes:** a mirror returning wrong content → **rejected** by local CID verification.
- **Provenance forgery:** an unsigned/misattributed publish → **rejected**; a stolen token can't mutate an existing version.

Each red‑team case maps to the security control that defeats it (DESIGN §13) and to a `SEC-*` backlog task.

## 3. What "green" gates

- **A task** merges only when its conformance slice + repro (if core) + red‑team slice + its spec update are all green and an independent Verifier signed off (Playbook §4).
- **A phase** exits only when its exit‑gate evidence (ROADMAP) is demonstrably met — a passing suite, a load report, an audit close‑out — reviewed by the human steering group.
- **A regression anywhere** blocks merge; there is no `skip`‑to‑pass and no "fix later" merge. A flaky test is quarantined *and* ticketed as a bug, because flakiness undermines mechanical acceptance.

## 4. Test data & environments

- **Fixtures** live in `conformance/` and are content‑addressed themselves (dogfooding), so a fixture can't silently drift.
- **The npm corpus** (`corpus/`) is pinned by CID snapshots so the compatibility baseline is reproducible run‑to‑run.
- **Ephemeral registries** spin up per‑e2e‑run (name+log+content services) so publish/verify tests are hermetic and parallelizable.
- **Two‑machine repro** is achieved with a matrix runner; the harness fails if the second machine's hashes differ from the first.

## 5. Observability of the test system itself

The Orchestrator treats CI as the source of truth (Playbook §3). ⟦Bnpmboard⟧s track: conformance coverage (gates green / total), repro pass rate, corpus pass‑rate vs. npm baseline over time, red‑team corpus size (must be monotonic), and flake rate (must trend to zero). A drop in any is a standing task, not an ignored metric.

## 6. Release gates (M7)

Before GA, three independent gates must close: **SLOs met under load** (resolve p99, fetch throughput, publish latency — targets set in M3, re‑verified under M7 load), **external security audit closed with no criticals**, and **the full conformance + repro + red‑team + corpus suites green** on the release commit. Only then does the steering group ratify the frozen 1.0 spec.
