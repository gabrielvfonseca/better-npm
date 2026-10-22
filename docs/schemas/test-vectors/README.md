# Bnpm — Test Vectors

> **Status:** normative · **Design refs:** ARCH-M004, CONTENT-ADDRESSING.md · **Conformance:** `conformance/cid/`

**Audience:** implementers, verifiers

---

Test vectors are the **executable specification** of Bnpm's core algorithms. An implementation passes a conformance gate when it produces the expected output for every vector in this directory.

## Directory layout

```
test-vectors/
├── README.md              # This file
├── cid/
│   └── cid-vectors.json   # CID construction: canonical bytes → b3: string
└── ...                    # Future: capsule, manifest, resolver, DPR
```

## Vector format

Every vector file follows a consistent schema:

```json
{
  "$schema": "../test-vector-schema.json",
  "specRef": "docs/specs/<spec>.md",
  "description": "Human-readable description of what this file tests",
  "version": 1,
  "vectors": [
    {
      "inputDescription": "What this specific vector exercises",
      "canonicalBytes": "68656c6c6f",
      "expectedCid": "b3:d4qovdywhwzynauslzcjdrpfrvf3gudo7dau5n4kq3uqrrlcjjtsady"
    }
  ]
}
```

### Common fields

| Field | Type | Description |
|---|---|---|
| `$schema` | string | Relative path to the schema file for this vector set |
| `specRef` | string | Path to the normative spec this vector tests |
| `description` | string | Set-level description of what is being tested |
| `version` | integer | Schema version (incremented on breaking changes) |
| `vectors[]` | array | Array of test cases |

### Per-vector fields

| Field | Type | Required | Description |
|---|---|---|---|
| `inputDescription` | string | yes | Human-readable description of what the vector tests |
| `canonicalBytes` | string | yes | Input bytes as lowercase hex string (no `0x` prefix). Empty string `""` is valid. |
| `expectedCid` | string | yes | Expected output: `b3:` prefix + lowercase RFC 4648 base32 with no padding |
| `blake3DigestHex` | string | no | Intermediate BLAKE3-256 digest (for debugging, not consumed by conformance) |
| `multihashHex` | string | no | Intermediate multihash bytes (for debugging, not consumed by conformance) |

## How to use

### In a conformance harness

1. Read the vector file.
2. For each vector:
   a. Parse `canonicalBytes` from hex to raw bytes.
   b. Compute the CID using the algorithm in [`docs/specs/CONTENT-ADDRESSING.md`](../../specs/CONTENT-ADDRESSING.md).
   c. Assert `computed_cid === vector.expectedCid`.
3. All vectors MUST pass. Any failure is a spec deviation that must be resolved before merge.

### Independent verification

A second, independent implementation (even a minimal script) MUST reproduce the exact `expectedCid` values in this file. This is the cross-implementation conformance mechanism described in [`docs/plan/TESTING-STRATEGY.md`](../../plan/TESTING-STRATEGY.md) §2.2.

### Adding vectors

1. Add your vector to the end of the relevant `*-vectors.json` array.
2. All fields are required (`inputDescription`, `canonicalBytes`, `expectedCid`).
3. Include the intermediate `blake3DigestHex` and `multihashHex` fields for debugging — they help implementers locate the source of a mismatch.
4. Vectors are sorted by conceptual complexity (simple → edge case → large) but new entries append to the end to avoid renumbering.

## Generating new vectors

Use BLAKE3 reference implementation + RFC 4648 base32:

```python
import blake3
import base64

def compute_cid(canonical_hex):
    canonical_bytes = bytes.fromhex(canonical_hex)
    digest = blake3.blake3(canonical_bytes).digest()
    multihash = bytes([0x1f, 0x20]) + digest
    b32 = base64.b32encode(multihash).decode('ascii').lower().rstrip('=')
    return f"b3:{b32}"
```

---

*Back to the [specifications index](../../specs/README.md).*
