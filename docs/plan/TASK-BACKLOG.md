# Bnpm — Task Backlog

The executable work list. Agents pull tasks from here; the Orchestrator advances them through the lifecycle in [`AGENT-PLAYBOOK.md`](AGENT-PLAYBOOK.md). Structure: **Epic → Task**. Each task has an **ID**, the **workstream/role** that owns it, **depends‑on** IDs, and **acceptance criteria** (its local Definition of Done, on top of the global one in the Playbook §4).

**ID scheme:** `<WS>-<phase><nn>` e.g. `CORE-M101`. **Role key:** IMPL = implementer, VERIFY = verifier, SEC = security, DOC = doc, QA = qa, INT = integration, ARCH = architect, ORCH = orchestrator.

> This backlog is the *seed* set — the epics and the critical‑path tasks through M6, plus M7 epics. During execution the Orchestrator decomposes each epic further; a task that turns out larger than ~1 focused work session is split. Every task implies a paired VERIFY task even where not listed separately.

---

## M0 — Spec‑lock

### Epic E0.1 — Ratify contracts
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| ARCH-M001 | Freeze `DESIGN.md` at RFC‑0001; open the change‑control (RFC) process | ARCH | — | Steering group ratifies; doc tagged normative |
| ARCH-M002 | Author JSON Schema for `bnpm.toml`, `bnpm.lock`, `[ai]` block, `bnpm.policy.toml` | ARCH | ARCH-M001 | Schemas validate their own examples in CI |
| ARCH-M003 | Author CBOR schema for the `BNPMCAP1` capsule + canonical manifest encoding | ARCH | ARCH-M001 | Round‑trips a fixture tree deterministically |
| ARCH-M004 | Specify CID construction (BLAKE3 multihash, canonicalization) + publish test vectors | ARCH | ARCH-M001 | Vectors reproducible by an independent impl |
| ARCH-M005 | Specify DPR message schemas (`RESOLVE`/`FETCH`/`PUBLISH`) | ARCH | ARCH-M001 | Schemas complete with error model |

### Epic E0.2 — Conformance skeleton & CI
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| QA-M001 | Stand up `conformance/` with every required behavior as a failing/`skip` test | QA | ARCH-M002..005 | `bnpm conformance --list` enumerates all gates |
| QA-M002 | Wire CI lanes: build · unit · conformance · reproducibility · red‑team · e2e | QA | QA-M001 | All lanes required; red on unimplemented slices |
| DOC-M001 | Publish `GLOSSARY.md` + ship all schema files next to their specs | DOC | ARCH-M002..005 | One term/one meaning; schemas discoverable |
| DOC-M002 | Extract the ✍️ spec docs from `DESIGN.md` (stubs with headers + design‑refs) | DOC | ARCH-M001 | Each row in README §2–§6 has a file + header block |

**M0 exit gate:** schemas ratified, conformance skeleton enumerates all gates, CI green on "schemas validate examples."

---

## M1 — Content core

### Epic E1.1 — CID & canonical encoding (`bnpm-cid`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| CORE-M101 | Implement BLAKE3 CID + multihash + `b3:` base32 formatting | IMPL | ARCH-M004 | Matches published vectors byte‑for‑byte |
| CORE-M102 | Implement canonical CBOR encoder (sorted, normalized, no float ambiguity) | IMPL | CORE-M101 | Same input → same bytes across platforms |
| CORE-M103V | Verify `bnpm-cid` (cross‑platform vectors, fuzz) | VERIFY | CORE-M101,102 | Fuzz finds no nondeterminism; vectors pass |

### Epic E1.2 — Capsule format (`bnpm-capsule`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| CORE-M104 | Implement `BNPMCAP1` packer (header, sorted file table, zstd objects, normalized metadata) | IMPL | CORE-M102, ARCH-M003 | Two packs of one tree → identical CID |
| CORE-M105 | Implement unpacker + per‑file range read using the file table + BLAKE3 range proofs | IMPL | CORE-M104 | Fetch‑3‑of‑200 modules verified without full download |
| CORE-M106 | Detached attestation section (add/rotate without changing content CID) | IMPL | CORE-M104 | Signature added; content CID unchanged |
| CORE-M107V | Verify capsule determinism + range‑read integrity (fuzz the packer) | VERIFY | CORE-M104..106 | Repro harness green; fuzz clean |

### Epic E1.3 — Store & materialization (`bnpm-store`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| CORE-M108 | CAS layout (`cas/`, `files/`, `unpacked/`, `index.db`) + capsule/file dedup | IMPL | CORE-M105 | Identical files across versions stored once |
| CORE-M109 | Materialization chain: reflink → hardlink → copy, per‑FS detection | IMPL | CORE-M108 | Zero‑copy on APFS/Btrfs; correct fallback |
| CORE-M110 | `bnpm verify` (local): prove stored bytes match their CID | IMPL | CORE-M108 | Tampered byte detected; offline |
| QA-M101 | Reproducibility harness: pack/store/materialize a 10k‑file corpus on 2 OSes | QA | CORE-M104,108 | Byte‑identical capsules + trees cross‑OS |

**M1 exit gate:** pack→store→materialize→verify round‑trips; cross‑platform byte‑identical; zero‑copy confirmed.

---

## M2 — Resolve & install

### Epic E2.1 — Manifest (`bnpm-manifest`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| RES-M201 | Parse/validate `bnpm.toml` against schema; typed `exports`; capability blocks | IMPL | ARCH-M002 | Rejects invalid; round‑trips valid |
| RES-M202 | Import `package.json` → in‑memory manifest (compat) | IMPL | RES-M201 | An npm project reads with no `bnpm.toml` |

### Epic E2.2 — Resolver (`bnpm-resolver`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| RES-M203 | MVS core (lowest‑version‑satisfying, max‑of‑minimums walk) | IMPL | RES-M201, CORE-M108 | Matches oracle on labeled graphs; terminates |
| RES-M204 | Singleton unification (`singleton=true`) + graph‑local multi‑version coexistence | IMPL | RES-M203 | Two‑React case unifies or fails with reason |
| RES-M205 | Bounded PubGrub fallback with human/agent‑readable conflict explanations | IMPL | RES-M203 | Real conflict → precise "cannot satisfy" msg |
| RES-M206 | Semver‑honesty checker: API‑surface diff flags breaking minor/patch | IMPL | RES-M201 | Flags planted breakage on labeled corpus |
| RES-M207V | Verify resolver (oracle diff, determinism, explanation quality) | VERIFY | RES-M203..206 | Byte‑identical results; every edge explained |

### Epic E2.3 — Lockfile & install (`bnpm-lock`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| RES-M208 | `bnpm.lock` writer (canonical order, CIDs, self‑digest) + reader | IMPL | RES-M203, CORE-M101 | No‑op re‑resolve → byte‑identical lock |
| RES-M209 | Strict non‑hoisted linker into `node_modules` | IMPL | RES-M208, CORE-M109 | No phantom deps; toolchain sees a real tree |
| RES-M210 | `bnpm install/add/up/remove/why` + `--offline` + `--json`/`--why` | IMPL | RES-M209 | Warm install = 0 network, bit‑identical to cold |
| RES-M211 | Workspace graph (one lock, one store view, cross‑pkg dedup) | IMPL | RES-M208 | 500‑pkg monorepo installs in one pass |
| QA-M201 | Golden‑file CLI tests + cross‑machine lock determinism | QA | RES-M210 | Same lock on every machine |

**M2 exit gate:** deterministic lock cross‑machine; zero‑I/O warm install; `--why` on everything; monorepo dedup.

---

## M3 — Registry MVP

### Epic E3.1 — Transport & content (`bnpm-proto`, `content-service`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| REG-M301 | DPR over HTTP/3 (QUIC) client+server with HTTP/2→HTTPS fallback | IMPL | ARCH-M005, CORE-M105 | Works behind a proxy via fallback |
| REG-M302 | `content-service`: serve capsules by CID; any mirror; local verification | IMPL | REG-M301 | Byte from untrusted mirror verified locally |
| REG-M303 | Delta transfer: send only changed file objects on upgrade | IMPL | REG-M302, CORE-M108 | 1.4.1→1.4.2 transfers only diffs |

### Epic E3.2 — Control plane (`name-service`, `log-service`, `bnpm-provenance`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| REG-M304 | `name-service`: name→version→CID pointers + tags | IMPL | REG-M301 | Immutable bindings; tags movable |
| REG-M305 | `log-service`: append‑only Merkle transparency log + inclusion proofs | IMPL | CORE-M101 | Proof verifiable offline |
| REG-M306 | Witness cosigning of log checkpoints (split‑view detection) | SEC | REG-M305 | Simulated split view detected |
| REG-M307 | `bnpm-provenance`: Sigstore/OIDC keyless signing + in‑toto/SLSA attest | SEC | REG-M305 | Mandatory; CI OIDC publish, no long‑lived token |
| REG-M308 | `bnpm publish` (single) → verify → attest → bind → log → proof | IMPL | REG-M304,305,307 | End‑to‑end publish returns inclusion proof |
| REG-M309 | `bnpm verify` (remote): prove a locked CID is legitimately published | IMPL | REG-M305, RES-M208 | Offline‑after‑fetch verification |
| REG-M310 | Minimal web UI: browse package + provenance + log entry | IMPL | REG-M304,305 | Provenance visible per version |
| REG-M311V | Verify control plane e2e + witness/split‑view + load to interim SLO | VERIFY | REG-M301..310 | Clean‑machine publish→resolve→fetch→verify |

**M3 exit gate:** CI OIDC publish → logged w/ proof → resolve+fetch+verify from clean machine offline‑after‑fetch; split‑view detected.

---

## M4 — Security & capabilities

### Epic E4.1 — Hermetic builds (`bnpm-sandbox`, builders)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| SEC-M401 | Hermetic build sandbox: no net/fs/env/subprocess by default | SEC | CORE-M109 | Escape attempts fail; inputs/outputs declared |
| SEC-M402 | Declarative builders `bnpm:tsc`/`bnpm:wasm`/`bnpm:node-gyp` as capsules | IMPL | SEC-M401 | Deterministic outputs, content‑addressed |
| SEC-M403 | Global build‑output cache (identical inputs+toolchain → cache hit) | IMPL | SEC-M402, CORE-M108 | Native module becomes a verified download |

### Epic E4.2 — Capability engine & scanning
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| SEC-M404 | Capability model enforcement + runtime permission gen (Deno/Node) | SEC | RES-M201 | No‑net package genuinely cannot open a socket |
| SEC-M405 | Capability‑diff gating in resolve/install (block privilege escalation pending review) | SEC | SEC-M404, RES-M210 | Patch that adds `net` is blocked, explained |
| SEC-M406 | Publish‑time + continuous scanning: malware, secrets, typosquat/dep‑confusion | SEC | REG-M308 | Planted typosquat + secret caught; retro‑flag |
| SEC-M407 | `@better-npm/sbom`: SPDX/CycloneDX from the lock + SPDX license resolution + change detection | IMPL | RES-M208 | SBOM == lock exactly; relicense flagged |
| SEC-M408 | `policy-service` + `bnpm.policy.toml` + `bnpm audit`/`bnpm policy check` | IMPL | SEC-M405,407 | Copyleft‑in‑proprietary fails CI with reason |
| SEC-M409V | Red‑team pass: worm‑class, dep‑confusion, install‑exploit, capability‑escalation | VERIFY | SEC-M401..408 | All planted attacks blocked or nowhere to run |

**M4 exit gate:** privilege‑escalating release blocked; install‑time exploit has no execution point; SBOM exact; policy gates CI.

---

## M5 — npm bridge & migration

### Epic E5.1 — Compatibility bridge (`npm-compat`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| BRIDGE-M501 | npm tarball → canonical capsule → logged CID ("sourced from npm" provenance) | IMPL | CORE-M104, REG-M305 | Stable CID; recorded in log |
| BRIDGE-M502 | npm registry as a federated upstream (resolve/fetch delegation) | IMPL | BRIDGE-M501, RES-M203 | npm dep resolves through Bnpm |
| BRIDGE-M503 | `node_modules` compatibility mode (strict link tree) for legacy tooling | IMPL | RES-M209 | Real toolchain runs unmodified |
| BRIDGE-M504 | Legacy lifecycle‑script quarantine (drop or hermetic‑sandbox only) | SEC | SEC-M401 | Imported script gains no ambient authority |
| BRIDGE-M505 | Dual‑publish (`bnpm publish` → Bnpm registry + npm tarball) | IMPL | REG-M308 | One command reaches both audiences |

### Epic E5.2 — Compatibility corpus (`corpus/`)
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| BRIDGE-M506 | Corpus harness: install + run each target's own test suite through Bnpm | INT | BRIDGE-M502,503 | Automated pass‑rate report vs. npm baseline |
| BRIDGE-M507 | Run top‑1000 by downloads; triage failures to owning components | INT | BRIDGE-M506 | ≥ npm pass rate; failures ticketed |
| BRIDGE-M508 | End‑to‑end migration of a real medium app via the 5‑step path | INT | BRIDGE-M503 | Step 1 needs zero code changes |

**M5 exit gate:** top‑1000 install + pass own tests at ≥ npm rate; `package.json`‑only works; dual‑publish works.

---

## M6 — AI‑native surface

### Epic E6.1 — Machine‑readable substrate
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| AI-M601 | API‑surface extraction from published source (types, signatures, shape) | IMPL | RES-M201 | Canonical per‑version API model |
| AI-M602 | Version‑to‑version API diffing (feeds semver‑honesty + `plan-upgrade`) | IMPL | AI-M601 | Accurate on held‑out corpus |
| AI-M603 | Verified‑examples runner (sandbox each example vs. the real version) | IMPL | SEC-M401, AI-M601 | Pass/fail recorded per example/version |
| AI-M604 | Tool‑interface (`ai/tools.json`) validation + indexing (MCP‑compatible) | IMPL | ARCH-M002 | Query the capability space by tool shape |

### Epic E6.2 — Index & agent endpoints
| ID | Task | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| AI-M605 | `index-service`: lexical + structural + vector search with explainable ranking | IMPL | AI-M601, REG-M304 | Constraint query returns ranked, cited hits |
| AI-M606 | `/.well-known/bnpm/` endpoints: `select`/`explain`/`plan-upgrade`/`audit`/`compose` | IMPL | AI-M605, SEC-M408, REG-M307 | Typed, cited; every claim links to the log |
| AI-M607 | `@better-npm/agent` SDK incl. transactional `apply` (edit manifest → regen lock → verify) | IMPL | AI-M606, RES-M210 | `apply` refuses on policy fail; lock verifies |
| AI-M608 | Scoped, time‑boxed agent identities + per‑action audit | SEC | REG-M307 | Agent can't accept new `subprocess` grant |
| AI-M609V | Verify: agent solves "add retrying HTTP client, MIT/Apache, no native, SLSA≥3" e2e | VERIFY | AI-M605..608 | PR lock `bnpm verify` passes; claims cited |

**M6 exit gate:** agent selects+installs+opens a verifiable, cited PR; `plan-upgrade` API‑diff impact accurate on held‑out data.

---

## M7 — Platform & GA (epics)

| ID | Epic | Role | Depends‑on | Acceptance |
|---|---|---|---|---|
| PLAT-M701 | Federation + private/self‑hosted registry with local policy at the boundary | IMPL | REG-M304, BRIDGE-M502 | Private+federated; reads federate, policy enforced |
| PLAT-M702 | Governance: orgs/teams/roles/capability ACL + full audit log | IMPL | REG-M307 | Every grant/publish/yank/policy decision audited |
| PLAT-M703 | Reputation scoring (multi‑signal, Sybil‑resistant, inspectable inputs) | IMPL | SEC-M406, AI-M605 | Weights verified signals over spoofable ones |
| PLAT-M704 | Observability/analytics dashboards from privacy‑gated telemetry | IMPL | REG-M304 | Dep‑health, upgrade‑lag, exposure‑over‑time |
| PLAT-M705 | Extensibility: registry plugins (sandboxed), builder capsules, webhooks/event stream | IMPL | SEC-M401, REG-M308 | Third‑party scanner/policy plugs in safely |
| PLAT-M706 | Performance hardening to published SLOs (resolve p99, fetch tput, publish latency) | QA | REG-M311 | SLOs met under load test |
| PLAT-M707 | External security audit + close‑out | SEC | SEC-M409 | No criticals open |
| PLAT-M708 | Freeze Bnpm 1.0 spec + compatibility promise | ARCH | all | Steering group ratifies 1.0 |

**M7 exit gate (GA):** SLOs met under load; audit closed no‑criticals; ≥1 external org in production; 1.0 ratified.

---

## Cross‑cutting standing tasks (never "done")

| ID | Task | Role | Cadence |
|---|---|---|---|
| QA-STANDING | Grow the reproducibility + red‑team + conformance suites monotonically | QA/SEC | every phase |
| DOC-STANDING | Keep ✍️ specs ≡ behavior (CI‑enforced design‑refs↔code map) | DOC | every PR |
| SEC-STANDING | Threat‑model review at each phase gate; triage new CVE intel to retro‑scanning | SEC | each gate |
| ORCH-STANDING | Keep the board truthful; advance waves; decompose oversized tasks; route blockers | ORCH | continuous |

---

## Spike tasks (de‑risk the DESIGN §23 unknowns early)

Run these *before* committing the dependent workstream, so the plan probes its riskiest bets first (BUILD‑PLAN §8).

| ID | Spike | Answers the risk |
|---|---|---|
| SPIKE-01 | Bit‑reproducible TS→JS transpile across TS versions/runtimes | "publish‑source" determinism (DESIGN §7.2, §23) |
| SPIKE-02 | Runtime capability enforcement matrix: Deno vs Node vs workers | uneven runtime enforcement (§13.3, §23) |
| SPIKE-03 | Semver‑honesty accuracy on dynamic/`any`‑heavy TS | MVS trustworthiness (§9.1, §23) |
| SPIKE-04 | Native‑module (node‑gyp/prebuilt) → hermetic content‑addressed build | native long‑tail migration (§23) |
| SPIKE-05 | Reputation model resistance to Sybil/download‑gaming | reputation governance (§19.4, §23) |
