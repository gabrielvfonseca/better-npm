# Dash — Schema Definitions

This directory contains machine-readable schema files for the Dash package format and its canonical CBOR encoding.

## Files

| File | Format | Purpose |
|---|---|---|
| `dash.capsule.cddl` | CDDL (RFC 8610) | Binary framing of the `DASHCAP1` capsule container: header, file table, file objects, attestations |
| `manifest.cddl` | CDDL (RFC 8610) | Canonical CBOR encoding of the Dash manifest (`dash.toml`) inside the capsule |

## Schema layers

```
dash.toml  ──(canonical TOML→CBOR)──►  manifest.cddl
                                              │
                                              ▼
                                    ┌──────────────────┐
                                    │ DASHCAP1 capsule  │
                                    │   ┌────────────────┐│
                                    │   │ Header (magic  ││
                                    │   │ + version      ││
                                    │   │ + flags)       ││
                                    │   ├────────────────┤│
                                    │   │ Manifest CBOR  ││◄── manifest.cddl
                                    │   ├────────────────┤│
                                    │   │ File table     ││◄── dash.capsule.cddl
                                    │   ├────────────────┤│
                                    │   │ File objects   ││
                                    │   │ (zstd)         ││
                                    │   ├────────────────┤│
                                    │   │ Attestations   ││
                                    │   └────────────────┘│
                                    └──────────────────────┘
```

## Relationship to JSON Schema

Dash uses **two schema regimes** for two distinct validation contexts:

| Aspect | JSON Schema | CDDL |
|---|---|---|
| **Audience** | `dash.toml` authoring (editors, LSPs, human/agent validation) | Capsule binary format (implementers, parsers, serializers) |
| **Format validated** | TOML (parsed as equivalent JSON values) | Canonical CBOR (inside the capsule) |
| **Location** | `packages/*/schemas/` (shipped with SDKs) | `docs/schemas/` (specification reference) |
| **Validation tooling** | Any JSON Schema validator | CBOR-native validators / `cddlc` / Rust `ciborium` + `cddl` |

The JSON Schema files describe the **authoring contract** — what a valid `dash.toml` looks like — while the CDDL files describe the **wire/storage format** — the exact byte representation inside the capsule. Both are derived from the same normative source (DESIGN §6, §7) and must not contradict each other. A manifest that validates against JSON Schema will, when canonically encoded as CBOR, validate against `manifest.cddl`.

## Canonical encoding rules (manifest.cddl)

The canonical CBOR encoding within the DASHCAP1 capsule follows RFC 8949 §4.2.1 (deterministic CBOR):

1. **Integer encoding:** encoded in the smallest form (e.g., 0–23 in one byte)
2. **Map key ordering:** keys sorted lexicographically by UTF-8 byte sequence
3. **String encoding:** definite-length strings only, no indefinite-length
4. **Floats:** encoded in the smallest representation that preserves value
5. **No undefined values:** CBOR `undefined` is never used; absent TOML sections produce absent CBOR map keys

These rules guarantee that the same `dash.toml` content always produces identical CBOR bytes on any platform, which is essential for deterministic CID computation.

## Capsule determinism (dash.capsule.cddl)

The full DASHCAP1 capsule achieves byte-level determinism through:

- **Sorted file table:** entries ordered lexicographically by path (UTF-8 byte order)
- **Normalized metadata:** fixed `mtime = 0`, fixed permission class (not raw mode bits), no uid/gid
- **Fixed compression:** zstd at a predetermined compression level (level 3, the default)
- **Deterministic manifest CBOR:** per canonical CBOR encoding rules above
- **No timestamps or random data in headers:** the header version and flags are the only varying fields, and they are set to known values

## References

- [RFC 8610](https://datatracker.ietf.org/doc/html/rfc8610) — CBOR Data Definition Language (CDDL)
- [RFC 8949](https://datatracker.ietf.org/doc/html/rfc8949) — CBOR (Concise Binary Object Representation)
- DESIGN §5 — Content-addressable storage and CID construction
- DESIGN §6 — Manifest schema
- DESIGN §7 — Package format (capsule container)
- DESIGN §9 / App. C — Canonical encoding and resolution algorithm
