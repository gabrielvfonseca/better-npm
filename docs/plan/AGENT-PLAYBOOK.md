# Dash — Agent Playbook

**How a fleet of AI engineering agents (supervised by humans) actually builds Dash.**

Companion to [`BUILD-PLAN.md`](BUILD-PLAN.md) (what to build) and [`TASK-BACKLOG.md`](TASK-BACKLOG.md) (the units of work). This document is the *operating manual*: roles, the task lifecycle, coordination, guardrails, and the Definition of Done that lets work be accepted or rejected mechanically.

The plan assumes agents are capable but fallible, run concurrently, and must not be trusted blindly — the same stance Dash itself takes toward packages. So the process is built on **isolation, explicit interfaces, and independent verification**, not on any single agent being right.

---

## 1. Roles

Roles are hats, not headcount — one runtime agent may wear several over time, but never two at once on the same task, and **never both implementer and verifier of the same unit**.

| Role | Mandate | Key tools | Never does |
|---|---|---|---|
| **Orchestrator** | Owns the backlog and the DAG. Assigns ready tasks, enforces waves, merges accepted work, keeps the board truthful. | task board, CI status, DAG | write product code |
| **Architect** | Owns `DESIGN.md` and the M0 schemas. Ratifies interface RFCs. Resolves cross‑component ambiguity. | RFC process, schemas | implement components |
| **Implementer** | Builds exactly one component/task to its acceptance criteria. Owns its tests and its spec update. | worktree, editor, local test | touch another component's internals |
| **Verifier** | Independently checks a completed task against acceptance criteria; runs repro + red‑team slices; tries to break it. | conformance runner, fuzzers | fix the code it reviews |
| **Security agent** | Maintains the threat model and red‑team suite; reviews every capability/auth/provenance‑touching change. | red‑team corpus, SAST | approve its own findings' fixes |
| **Doc agent** | Keeps ✍️ specs ≡ behavior; owns `GLOSSARY.md`; validates schemas ship with examples. | docs, schema validators | change behavior |
| **Integration agent** | Runs the compatibility corpus (M5) and cross‑component end‑to‑end suites; triages failures to owning components. | corpus harness, e2e | patch components directly |

Humans sit above the Orchestrator and Architect as a **steering group**: they ratify phase‑exit gates, approve interface RFCs with ecosystem impact, and hold the keys for anything irreversible (production publish credentials, the transparency‑log root of trust, external releases).

---

## 2. The task lifecycle

Every unit of work moves through a fixed state machine. The Orchestrator advances states; agents do the work inside them.

```
BACKLOG → READY → CLAIMED → IN-PROGRESS → IN-REVIEW → {ACCEPTED | REJECTED} → MERGED
                                   │                        │
                                   └──── BLOCKED ◄──────────┘
```

1. **READY** — all `depends-on` tasks are `MERGED` and the interfaces it consumes are frozen. Only the Orchestrator moves `BACKLOG→READY`.
2. **CLAIMED** — exactly one Implementer claims it; ownership is exclusive for its component.
3. **IN‑PROGRESS** — the Implementer works in an **isolated worktree**, writing code + tests + the spec update together. Progress is visible via the task board, not narration.
4. **IN‑REVIEW** — the Implementer opens a PR and hands off to a **different** agent (Verifier, plus Security agent if the change touches security‑sensitive surface). The implementer does not self‑merge.
5. **ACCEPTED / REJECTED** — the Verifier runs the Definition of Done (§4) mechanically. Rejection returns the task to `IN‑PROGRESS` with specific, reproducible failures — never a vibe.
6. **MERGED** — the Orchestrator merges and unblocks downstream tasks in the DAG.

**BLOCKED** is a first‑class state: an agent that discovers a missing interface, an ambiguous spec, or a cross‑component need raises a blocker (with the exact question) and stops rather than guessing or reaching into a neighbor. The Orchestrator routes blockers to the Architect (spec/interface) or schedules the missing dependency.

---

## 3. Coordination protocol (how agents avoid stepping on each other)

The four rules that make concurrency safe:

1. **Single‑writer components.** At any moment, one component has at most one Implementer. Cross‑cutting changes are decomposed into per‑component tasks, each single‑writer.
2. **Interfaces before implementations.** No agent may depend on another's *implementation*; it depends only on the *frozen interface* (M0 schema / signature). If the interface it needs doesn't exist, that's an interface RFC to the Architect, not a peek at someone's WIP.
3. **Isolated worktrees + PR integration.** All mutation happens in per‑task worktrees. The only integration point is the PR/merge queue, which serializes conflicting merges. Agents never share a working directory.
4. **The board is the single source of truth.** State lives on the task board and in CI, not in any agent's memory. An agent resuming work re‑reads the board; it never assumes prior context is still valid. (Mirrors Dash's own "verify, don't trust.")

Communication is **asynchronous and artifact‑mediated**: agents coordinate through tasks, PRs, RFCs, and conformance results — durable artifacts a returning agent or a human can audit — rather than through ephemeral chatter.

---

## 4. Definition of Done (the acceptance gate)

A task is **ACCEPTED** only if a Verifier confirms *all* of these. This is intentionally mechanical so acceptance is not a judgment call.

- [ ] **Conformance green** — the task's slice of `conformance/` passes; new behavior added new tests (published as reusable vectors where cross‑impl).
- [ ] **Determinism** — for data‑plane/core code, the reproducibility harness shows byte‑identical output across two OSes; no wall‑clock/random/locale/iteration‑order sources (lint‑enforced).
- [ ] **Interface fidelity** — public surface matches the frozen schema; any interface change went through a ratified RFC.
- [ ] **Security clean** — the red‑team slice passes; no new ambient authority; the Security agent signed off if the change touches sandbox/capabilities/auth/provenance.
- [ ] **Spec in lockstep** — the owning ✍️ spec doc and machine‑readable schema were updated in the same PR (CI enforces the `design-refs`↔code map).
- [ ] **DX contract** — user‑facing surfaces have `--json`, actionable errors, `--why` where meaningful, and golden‑file output tests.
- [ ] **Independent verification** — signed off by an agent other than the implementer.
- [ ] **No regression** — the full CI (build, unit, conformance, repro, red‑team, e2e where affected) is green.

If any box is unchecked, the task is **REJECTED** with the failing artifact attached. There is no partial credit and no "will fix later" merge.

---

## 5. Guardrails (what agents must and must not do)

**Must:**
- Work against frozen interfaces; raise a blocker when one is missing.
- Keep changes scoped to one component/task; open a new task for anything discovered out of scope.
- Write the test and the spec update *with* the code, never after.
- Prefer removing a concept over adding a flag (DESIGN §21) — reviewers reject unjustified surface area.
- Leave the tree green: never merge on a red suite, never `skip` a test to pass.

**Must not:**
- Self‑merge or self‑verify.
- Introduce nondeterminism into core crates.
- Add an install‑time script, a network call in a build, or any ambient authority — the system Dash is *building* forbids exactly what an agent might reflexively reach for.
- Weaken a security control to make a test pass (that's a blocker + a Security‑agent review, not a workaround).
- Touch production credentials, the transparency‑log trust root, or external publishing — those are human‑gated (§7).
- Fabricate a passing result. Acceptance is evidence‑based; a Verifier re‑runs, it does not take an implementer's word.

---

## 6. Handling failure, ambiguity, and loops

- **Ambiguous spec →** BLOCKED → Architect clarifies via a `DESIGN.md`/schema change (so the ambiguity is fixed *for everyone*, once).
- **Repeated CI failure (≥3 attempts on the same cause) →** stop, don't brute‑force. Escalate with what was tried and the exact failure; the Orchestrator may re‑scope or reassign.
- **Cross‑component bug found during integration →** the Integration agent files a task against the *owning* component; it does not patch across boundaries.
- **Flaky test →** quarantine + a task to fix the flake; flakes are treated as bugs, not noise, because a nondeterministic test undermines the mechanical acceptance gate.
- **Disagreement between agents →** resolved by the conformance suite; if the suite is silent, that's a missing test → a task to add it, decided by the Architect.

The meta‑rule mirrors the browser‑automation guidance the fleet already follows: **don't spin.** After a couple of honest attempts at a stuck problem, surface it with context instead of looping.

---

## 7. Human‑in‑the‑loop checkpoints

Agents run autonomously *within* a phase; humans gate *between* phases and on anything irreversible:

- **Phase‑exit gates** (M0…M7) — the steering group reviews the exit‑gate evidence before the next phase's tasks go `READY`.
- **Interface RFCs with ecosystem impact** — schema changes that affect published artifacts or compatibility need human ratification.
- **Roots of trust** — the transparency‑log signing key, witness onboarding, Sigstore trust config, and production publish credentials are human‑held and never delegated to an agent identity.
- **External releases** — tagging a public version, the security‑audit sign‑off (M7), and the 1.0 spec freeze are human decisions.

Everything an agent does inside these bounds is auditable: every change is a PR, every acceptance is evidence, every merge is attributable to an identity — the same provenance discipline Dash applies to packages, applied to its own construction.

---

## 8. Bootstrapping the fleet (day one)

A concrete cold‑start sequence the Orchestrator runs to go from empty repo to flowing work:

1. **Architect** lands M0: `DESIGN.md` ratified, all schemas + conformance skeleton committed (everything failing/`skip`). Steering group ratifies.
2. **QA agent** wires CI: build + unit + conformance + repro + red‑team lanes, all required, all currently red on unimplemented slices.
3. **Orchestrator** marks the two DAG roots — `dash-cid` and `dash-manifest` — `READY` and assigns Implementers.
4. **Doc agent** publishes the `GLOSSARY` and the JSON/CBOR schema files so downstream agents code against them.
5. Work flows: as each component goes green, the Orchestrator advances the wave table in [BUILD-PLAN §6](BUILD-PLAN.md) and fans out newly‑unblocked leaves.
6. **Verifier** and **Security** agents shadow every completion; nothing merges without independent sign‑off.

From here the loop is self‑sustaining: `READY → done → unblock → READY`, phase‑gated by humans, until GA.
