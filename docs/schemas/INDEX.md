# Dash — Schema Index

> **Status:** ratified · **Owner:** WS-DOC · **Design refs:** DOC-M001 · **Conformance:** n/a

This index lists every machine-readable schema file in this directory and which specification document(s) it serves.

## Schema registry

| Schema File | Format | Serves Spec Doc(s) | Status |
|---|---|---|---|
| `dash.toml.json` | JSON Schema (draft-07) | [MANIFEST.md](../docs/specs/MANIFEST.md) | ratified |
| `ai.block.json` | JSON Schema (draft-07) | [MANIFEST.md](../docs/specs/MANIFEST.md), [AI-NATIVE.md](../docs/specs/AI-NATIVE.md) | ratified |
| `dash.lock.json` | JSON Schema (draft-07) | [LOCKFILE.md](../docs/specs/LOCKFILE.md) | ratified |
| `dash.capsule.json` | JSON Schema (draft-07) | [CAPSULE-FORMAT.md](../docs/specs/CAPSULE-FORMAT.md) | ratified |
| `dash.capsule.cddl` | CDDL (RFC 8610) | [CAPSULE-FORMAT.md](../docs/specs/CAPSULE-FORMAT.md) | ratified |
| `manifest.cddl` | CDDL (RFC 8610) | [CAPSULE-FORMAT.md](../docs/specs/CAPSULE-FORMAT.md) | ratified |
| `dpr-messages.json` | JSON Schema (draft-2020-12) | [PROTOCOL-DPR.md](../docs/specs/PROTOCOL-DPR.md), [REGISTRY-API.md](../docs/specs/REGISTRY-API.md) | ratified |
| `dash.policy.toml.json` | JSON Schema (draft-07) | [SECURITY-MODEL.md](../docs/specs/SECURITY-MODEL.md) | ratified |

## Conventions

- **JSON Schema** files describe the authoring contract — what a valid `dash.toml`, `dash.lock`, etc. looks like when parsed as JSON-equivalent values.
- **CDDL** files describe the wire/storage format — the exact CBOR byte representation inside the DASHCAP1 capsule.

## Related

- [README.md](./README.md) — Schema design rationale and encoding rules
- [Test vectors](./test-vectors/) — Executable conformance vectors

---

*Back to the [documentation index](../README.md).*
