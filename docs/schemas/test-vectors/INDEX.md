# Dash — Test Vector Index

> **Status:** ratified · **Owner:** WS-DOC · **Design refs:** DOC-M001 · **Conformance:** n/a

This index lists every test-vector directory and what it tests. Test vectors are the executable specification of Dash's core algorithms — an implementation passes a conformance gate when it produces the expected output for every vector.

## Vector directories

| Directory / File | Tests Spec | Status |
|---|---|---|
| [`cid/`](./cid/) — `cid-vectors.json` | [CONTENT-ADDRESSING.md](../../docs/specs/CONTENT-ADDRESSING.md) §2 — CID construction | ratified |
| (future: `capsule/`) | [CAPSULE-FORMAT.md](../../docs/specs/CAPSULE-FORMAT.md) — DASHCAP1 container | planned |
| (future: `manifest/`) | [MANIFEST.md](../../docs/specs/MANIFEST.md) — dash.toml schema validation | planned |
| (future: `resolver/`) | [RESOLVER.md](../../docs/specs/RESOLVER.md) — MVS + PubGrub resolution | planned |
| (future: `dpr/`) | [PROTOCOL-DPR.md](../../docs/specs/PROTOCOL-DPR.md) — DPR wire protocol messages | planned |
| (future: `lockfile/`) | [LOCKFILE.md](../../docs/specs/LOCKFILE.md) — dash.lock parse/generate | planned |

## How to use

See [`README.md`](./README.md) for the full vector format specification and instructions for running vectors in a conformance harness.

---

*Back to the [documentation index](../../README.md).*
