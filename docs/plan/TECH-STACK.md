# Dash — Technology Choices

Rationale for the implementation stack. The governing rule is **"boring by default"**: choose the most proven, least surprising technology that meets the requirement, and spend the novelty budget only where Dash's thesis actually demands it (content addressing, capabilities, provenance). Every choice below is justified against a requirement, not a preference.

## 1. The one rule

> Prefer mature, widely‑deployed, well‑understood technology. Reserve innovation for the core theses. A package manager is critical infrastructure; its stack should be unremarkable so its *design* can be remarkable.

This is also a plan‑for‑agents rule: agents build more reliably on well‑documented, high‑corpus technologies, and reviewers verify them more easily.

## 2. Core client & performance‑critical crates — **Rust**

**Where:** `dash-cid`, `dash-capsule`, `dash-store`, `dash-resolver`, `dash-lock`, `dash-manifest`, `dash-proto`, `dash-sandbox`, `dash-provenance`, `dash-cli`.

**Why:**
- **Determinism & control.** Content addressing demands byte‑exact, allocation‑controlled, no‑hidden‑nondeterminism code. Rust gives explicit control without a GC's timing variability.
- **Performance is a feature** (Bun's lesson, DESIGN §1.1): a single fast native binary makes every command feel instant. Rust delivers native speed with fearless parallelism for the many‑small‑files access pattern.
- **Memory safety** in the most security‑critical software a developer runs. No buffer‑overflow class in the installer that touches every project.
- **First‑class WASM** so the same core powers the TS SDK (`@dash/core`) via WASM bindings — one implementation of the hard logic, consumed everywhere.
- **Cross‑platform single binary** — trivial distribution, no runtime dependency.

**Key libraries:** `blake3` (hashing), `zstd` (capsule compression), `serde`/`ciborium` (canonical CBOR), a QUIC/HTTP3 stack (`quinn`/`h3`), `sigstore`‑rs + `in‑toto` for provenance. Determinism lint is a custom `clippy`‑style rule banning nondeterministic sources in core paths.

## 3. Control‑plane services — **Rust services + Postgres + object storage**

**Where:** `name-service`, `log-service`, `content-service`, `index-service`, `policy-service`, `gateway`.

**Why:**
- **Rust services** keep one language across client and server, share the `dash-*` crates (no protocol drift between what the client speaks and what the server implements), and meet the throughput SLOs.
- **Postgres** for the small, authoritative control‑plane state (name→version→CID pointers, orgs/policy). It's the boring, correct choice: transactions, mature ops, nothing exotic. The control plane is deliberately *small* (DESIGN §4), so it does not need a fancy datastore.
- **Object storage (S3‑compatible) + CDN** for the data plane (capsule blobs). Content is immutable and addressed by hash, so it's a perfect fit for cheap, cacheable, mirror‑friendly object storage. The data plane is *untrusted by construction* (verified locally), so it can live anywhere.
- **Append‑only Merkle log** (`log-service`) follows the Go `sumdb`/Sigstore Rekor lineage — a proven design for tamper‑evident transparency logs — with independent **witness cosigners** for split‑view resistance.
- **Vector index** for semantic search: a standard vector store alongside a lexical index (the structural index is derived from extracted types). No bespoke database.

## 4. SDKs & AI surface — **TypeScript**

**Where:** `@dash/core` (WASM bindings), `@dash/registry`, `@dash/sbom`, `@dash/agent`.

**Why:** these are consumed by JS/TS developers, build tools, and agents — they must live natively in that ecosystem. `@dash/core` wraps the Rust core via WASM so the hard logic isn't reimplemented. `@dash/agent` exposes the typed `select`/`plan-upgrade`/`apply` surface (DESIGN §18) in the language its callers use, with MCP‑compatible tool definitions so it drops into existing agent tooling.

## 5. Transport — **HTTP/3 (QUIC), with graceful fallback**

Justified in DESIGN §11: multiplexing without head‑of‑line blocking fits the "hundreds of small metadata requests + many parallel range fetches" pattern, and 0‑RTT resumption makes warm reconnects cheap. **Fallback to HTTP/2 → HTTPS is mandatory** for restricted networks and proxies — the protocol must degrade, never fail, in enterprise environments. All content responses are self‑verifying, so the transport is a performance choice, not a trust boundary.

## 6. Cryptography — **standards, not invention**

- **BLAKE3** for content addressing (throughput + built‑in Merkle tree for range proofs, DESIGN §5.1).
- **Sigstore / keyless OIDC** for provenance signing — the same technology npm's provenance and JSR adopted, so it interoperates with the existing supply‑chain‑security world rather than forking it.
- **in‑toto / SLSA** for build attestations — the industry standard, so Dash provenance is consumable by existing verifiers (e.g., `cosign`).
- **No home‑grown crypto.** The novelty is in *what* we sign and log (mandatory, capability‑aware, witnessed), not in new primitives.

## 7. Manifest & lockfile formats

- **TOML** for `dash.toml` (human‑authored) — Cargo's proven choice: unambiguous, comment‑friendly, diff‑friendly, resistant to the deep nesting that ruined `package.json` (DESIGN §6).
- **Canonical CBOR** for the on‑the‑wire/in‑capsule manifest and for the capsule container — compact, deterministic, schema‑checked.
- **Deterministic text** for `dash.lock` — sorted, keyed, merge‑friendly (DESIGN §8), so humans can review diffs and merges don't conflict positionally.
- **JSON Schema** for every human‑facing schema and **CBOR schema** for binary formats, shipped as files so agents validate rather than parse prose.

## 8. Build, CI & repo

- **Monorepo**, dogfooding Dash workspaces once M2 lands (the project uses itself — the strongest possible compatibility test).
- **CI lanes** (Testing Strategy §1) run on a **multi‑OS matrix** because reproducibility is cross‑platform or it's nothing.
- **Sandboxing** for hermetic builds uses OS primitives (namespaces/seccomp on Linux, the sandbox facilities on macOS, job objects on Windows) plus WASM for portable builders — no ambient authority, by construction (DESIGN §7.3, §13.3).

## 9. Explicitly rejected choices (and why)

| Rejected | Why not |
|---|---|
| Node/TS for the core | GC timing variability, slower for the syscall‑heavy install path, weaker determinism guarantees; kept for the *edges* where its ecosystem is the point |
| A blockchain for the "immutable log" | A witnessed append‑only Merkle log (sumdb/Rekor lineage) gives tamper‑evidence and auditability without the cost, latency, and governance baggage |
| A bespoke datastore for content | Content is hash‑addressed immutable blobs — commodity object storage + CDN is strictly better and mirror‑friendly |
| Home‑grown signing/hashing | Interop with the existing supply‑chain ecosystem (Sigstore/SLSA/cosign) matters more than cleverness; reinventing crypto is a liability |
| Git‑as‑registry (SwiftPM style) | Couples availability/performance to arbitrary hosting; the content‑addressed data plane + CDN is faster and more resilient (DESIGN §1.1) |
| SAT solver as the default resolver | Nondeterminism and churny locks; MVS is the default, SAT/PubGrub only for real conflicts (DESIGN §9.1) |

The through‑line: **spend novelty on the theses (content addressing, capabilities, mandatory witnessed provenance, the AI‑native surface); everywhere else, pick the proven thing** — which is also what makes the system safe for a fleet of agents to build and for a human steering group to audit.
