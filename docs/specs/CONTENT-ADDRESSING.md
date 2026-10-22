# Bnpm — Content-Addressing Specification

> **Status:** normative · **Owner:** WS-ARCH · **Design refs:** DESIGN §5 · **Implements:** ARCH-M004 · **Conformance:** `conformance/cid/`

**Audience:** implementers, verifiers

---

## 1. Purpose

This document specifies the **Content ID (CID)** construction algorithm, the canonicalization rules that precede hashing, the global store layout, and the dedup and materialization strategies. Every system component that reads or writes content-addressed objects depends on this specification.

Two implementations that follow this document must produce **byte-identical CIDs** for identical source trees, on any platform, in any language.

---

## 2. CID construction

### 2.1 Format

```
cid = "b3:" || base32_lc_nopad( multihash( BLAKE3-256( canonical_capsule_bytes ) ) )
```

A CID is an ASCII string with a `b3:` prefix followed by the lowercase-base32-encoded (RFC 4648) multihash of the BLAKE3-256 digest of the canonical capsule bytes.

**Examples:**

```
b3:d4qk6e2jxh27tingubae32rw3teutg6lexe23qisw7gjve6k4qpteyq    // empty capsule
b3:d4qovdywhwzynauslzcjdrpfrvf3gudo7dau5n4kq3uqrrlcjjtsady    // capsule containing "hello"
b3:d4qpxhzhyjtbwci4qjkhla7xq4djtx76dzhxdyhjnkokz5rgazhbomi    // CBOR {a:1, b:2}
```

### 2.2 Pseudocode

```
function compute_cid(canonical_bytes: byte[]) -> string:
    // 1. Hash with BLAKE3-256
    digest = blake3_256(canonical_bytes)
    assert length(digest) == 32

    // 2. Construct multihash
    //    multihash  = varint(hash_code) || varint(digest_length) || digest
    //    code for BLAKE3-256 = 0x1f (31), length = 0x20 (32)
    multihash = [0x1f, 0x20] ++ digest               // 34 bytes

    // 3. Base32 (RFC 4648, lowercase, no padding)
    b32 = base32_encode(multihash)
    b32 = lowercase(b32)
    b32 = strip_trailing(b32, '=')

    // 4. Prepend prefix
    return "b3:" ++ b32
```

### 2.3 Multihash format

The multihash follows the [multiformats/multihash](https://multiformats.io/multihash/) specification:

```
<varint-hash-code> || <varint-digest-length> || <digest-bytes>
```

**BLAKE3-256 parameters:**

| Field | Value | Encoding |
|---|---|---|
| Hash code | `0x1f` (31 decimal) | Unsigned LEB128 varint; 31 < 128 → single byte `0x1f` |
| Digest length | `0x20` (32 bytes) | Unsigned LEB128 varint; 32 < 128 → single byte `0x20` |
| Digest | 32 bytes of raw BLAKE3-256 output | Big-endian bytes |

Total multihash wire length: **34 bytes** (1 + 1 + 32).

**Multihash table entry:**

| Code | Name | Description |
|---|---|---|
| `0x1f` | `blake3-256` | BLAKE3 with 256-bit (32-byte) output |

> The hash code `0x1f` is registered in the [multicodec table](https://github.com/multiformats/multicodec) for BLAKE3-256. Implementations MUST use this exact value.

### 2.4 Base32 encoding

**Standard:** RFC 4648 §6 ("Base 32 Encoding")

**Character set (lowercase):**

```
      Alphabet: abcdefghijklmnopqrstuvwxyz234567
      Padding:  none
```

**Rules:**

- Encode the 34-byte multihash using RFC 4648 base32.
- The output MUST be **lowercase** (letters `a`–`z`, digits `2`–`7`).
- Padding characters (`=`) MUST be **omitted** from the final output.
- The encoded output before stripping padding is always **55 characters** (272 bits ÷ 5 = 54.4 → 55 characters). After stripping padding: **55 characters** (no padding needed).

**Pseudo:**

```
b32 = base32_encode(multihash, alphabet=RFC4648_LOWER, padding=false)
```

### 2.5 BLAKE3-256 hash

**Algorithm:** [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) with 256-bit output length.

**Properties:**

- Tree-structured hashing: enables parallel hashing of large inputs and incremental verification of byte ranges via the BLAKE3 Merkle tree.
- Default output length is 32 bytes (256 bits). Implementations MUST request exactly 32 bytes.
- No key, no salt, no personalization string is used (the hash is a pure function of input bytes).
- Cross-platform: BLAKE3 is deterministic across all architectures (no endianness, no SIMD width differences in the output — these are internal implementation details invisible to the digest).

---

## 3. Canonicalization

The CID is computed over **canonical_capsule_bytes** — a deterministic, platform-independent byte representation. The capsule container format is specified in a separate document (see [CAPSULE-FORMAT.md](CAPSULE-FORMAT.md)), but the canonicalization rules that apply to its contents are defined here.

### 3.1 Canonical CBOR encoding

The manifest and file table within a capsule use Canonical CBOR ([RFC 7049 §3.9](https://tools.ietf.org/html/rfc7049#section-3.9) with additional constraints).

#### 3.1.1 Map key ordering

CBOR map entries MUST be sorted by key in ascending bytewise order of their **canonical CBOR encoding**. This is NOT Unicode code-point order — it is raw byte comparison of the key's CBOR encoding.

```
// Example: { "b": 2, "a": 1 } is NOT canonical
// Canonical form: { "a": 1, "b": 2 }

// Incorrect (unsorted):
a2                    # map(2)
  61 62              # text(1) "b"
  02                 # unsigned(2)
  61 61              # text(1) "a"
  01                 # unsigned(1)

// Correct (sorted by key encoding):
a2                    # map(2)
  61 61              # text(1) "a"
  01                 # unsigned(1)
  61 62              # text(1) "b"
  02                 # unsigned(2)
```

#### 3.1.2 Integer encoding

Integers MUST use the **smallest encoding** that represents the value:

| Value range | CBOR encoding | Bytes |
|---|---|---|
| 0–23 | major type 0, inline value | 1 |
| 24–255 | major type 0, 1-byte uint8 | 2 |
| 256–65535 | major type 0, 2-byte uint16 | 3 |
| 65536–4294967295 | major type 0, 4-byte uint32 | 5 |
| > 4294967295 | major type 0, 8-byte uint64 | 9 |
| -1 to -24 | major type 1, inline value | 1 |
| -25 to -256 | major type 1, 1-byte | 2 |
| ... and so on | | |

```
// Examples:
0  -> 0x00          (not 0x18 0x00)
23 -> 0x17          (not 0x18 0x17)
24 -> 0x18 0x18
100 -> 0x18 0x64
500 -> 0x19 0x01 0xf4
```

#### 3.1.3 String encoding

- Text strings (`major type 3`) MUST be valid UTF-8.
- Byte strings (`major type 2`) are used for binary data.
- String lengths MUST use the smallest encoding (same rules as integers above).

#### 3.1.4 Float encoding — FORBIDDEN

CBOR float types (half-precision `f6`, single-precision `f7`, double-precision `f8`) are **forbidden** in canonical Bnpm capsule data. All numeric values MUST use integer encodings.

Rationale: Float encoding introduces ambiguity (1.0 can be represented as `f9 3c 00`, `fa 3f 80 00 00`, or `fb 3f f0 00 00 00 00 00 00`), violating the determinism requirement. Bnpm does not store floating-point metadata.

Implementations MUST reject capsule data containing major type 7 (`f6`–`f8`) with a canonicalization error.

#### 3.1.5 Indefinite-length encoding

Indefinite-length arrays (`9f`) and maps (`bf`) are **forbidden**. All CBOR structures MUST use definite-length encoding.

#### 3.1.6 Tagged values

CBOR tags (`major type 6`) are **forbidden** in canonical capsule data with the exception of tag 258 (`self-described CBOR`, encoding `d9 01 02`), which is used only at the outermost capsule header.

### 3.2 File metadata normalization

File entries in the capsule file table MUST have normalized metadata:

- **mtime:** fixed to `0` (epoch)
- **permissions:** fixed by file class (executable: `0o755`, non-executable: `0o644`)
- **uid/gid:** set to `0`
- **File paths:** UTF-8, forward-slash separators, no trailing slash, no `.` or `..` segments, sorted lexicographically by path

### 3.3 Compression determinism

If capsule file objects are compressed (zstd), the compressor MUST use a fixed compression level. The default level is `3`.

### 3.4 Input validation summary

Implementations MUST validate that canonical capsule bytes satisfy all of the above before hashing. If any rule is violated, the implementation MUST reject the input with an error (not silently produce a wrong CID).

---

## 4. Store layout

### 4.1 Directory structure

The global content-addressed store lives at `~/.bnpm/store/`:

```
~/.bnpm/store/
├── cas/
│   └── b3/
│       └── <aa>/
│           └── <bb>/
│               └── <cid>.capsule      # Immutable capsule blobs (compressed)
├── files/
│   └── b3/
│       └── <aa>/
│           └── <bb>/
│               └── <filehash>         # Individual file objects (granular reuse)
├── unpacked/
│   └── <cid>/                         # Materialized directory trees, ready to link
│       ├── node_modules/
│       └── ...  (project tree)
└── index.db                           # Local content index (SQLite)
```

### 4.2 Shard layout

Capsules and file objects are stored under a two-level shard directory derived from the CID (or file hash):

```
shard = b3/<first two hex chars of hash>/<next two hex chars of hash>/
```

Example for CID `b3:d4qpxhzhyjtbwci4qjkhla7xq4djtx76dzhxdyhjnkokz5rgazhbomi`:

```
~/.bnpm/store/cas/b3/d4/qp/d4qpxhzhyjtbwci4qjkhla7xq4djtx76dzhxdyhjnkokz5rgazhbomi.capsule
```

The shard key is derived from the **raw BLAKE3 digest hex** (the inner 32 bytes), not from the base32 CID string.

Sharding prevents any single directory from containing more than ~65k entries at a uniform distribution (256 first-level × 256 second-level directories).

### 4.3 index.db

The SQLite index stores:

- `capsules(cid, size_bytes, blake3_digest, inserted_at)` — map CID to capsule location
- `files(hash, cid, path, size_bytes)` — per-file entries for dedup and lookup
- `refs(cid, refcount)` — reference counts for GC
- `unpacked(cid, project_path, materialized_at)` — materialization tracking

---

## 5. Dedup strategy

### 5.1 Capsule-level dedup

Two identical capsule blobs (same content, same compressed bytes) produce the same CID and are stored **once** in `cas/`, regardless of how many projects or versions reference them.

Identical capsule bytes arise from:
- The same package version installed in multiple projects
- Different packages that happen to produce identical file trees (rare but possible)
- Cross-publisher reproducibility: different maintainers publishing identical source produce identical CIDs

### 5.2 File-level dedup

The file table in a capsule lists each file's **BLAKE3-256 file hash**. Identical files across different capsules (e.g., an unchanged file between `lib@1.4.1` and `lib@1.4.2`, or the same `lodash.js` in different packages) are stored **once** in `files/`.

File-level dedup is the mechanism that makes upgrades near-free on disk: upgrading a package typically changes only a fraction of its files, and unchanged file objects are already present in the store.

### 5.3 Dedup chain

```
Capsule A ── file table ──► file hash x ──► files/b3/<shard>/<filehash>
Capsule B ── file table ──► file hash x ──► (same object, no new storage)
```

---

## 6. Materialization

### 6.1 Strategy (preference order)

When materializing a capsule's files into a project's `node_modules`, the implementation attempts, in order:

1. **Reflink / copy-on-write (CoW)** — on APFS (macOS), Btrfs/XFS (Linux), ReFS (Windows). Uses `FICLONE` ioctl on Linux, `copyfile(CLONE)` on macOS, `FSCTL_DUPLICATE_EXTENTS_TO_FILE` on Windows. Zero-cost for the duplicate: shares physical storage until one side modifies its copy.
2. **Hardlink** — pnpm's approach. Multiple project directories point to the same inode in the store. Read-only at the store level to prevent accidental mutation.
3. **Copy** — last resort for filesystems that support neither reflink nor hardlinks (e.g., some FUSE mounts, FAT32, WebDAV). Fallback copies bytes.

### 6.2 Detection

The implementation detects the best available strategy per-filesystem:

```
function materialize_strategy(store_path: string, target_path: string) -> enum:
    if filesystem_supports_reflink(store_path, target_path):
        return REFLINK                          // FICLONE / copyfile CLONE
    elif filesystem_supports_hardlink(store_path, target_path):
        return HARDLINK                         // link(2), checked cross-device
    else:
        return COPY                             // read + write
```

Cross-device hardlinks (store on one volume, project on another) MUST be detected at runtime and fall back to copy.

### 6.3 Store integrity

Files in `cas/` and `files/` are **read-only** (chmod `0o444` perms, or filesystem-level immutable flag where available). This prevents accidental mutation through hardlinks.

### 6.4 Garbage collection

Reference counts (`index.db.refs`) track how many projects/unpacked trees reference each capsule and file. A GC pass (`bnpm store gc`) removes objects with zero references.

---

## 7. Cross-platform determinism guarantees

A CID computed on any platform (Linux x86_64, macOS ARM64, Windows x86_64, any other) MUST produce identical output for identical `canonical_capsule_bytes`. The following table enumerates the sources of potential nondeterminism and how they are eliminated:

| Nondeterminism source | Mitigation |
|---|---|
| **File modification time** | Set to fixed value `0` in capsule |
| **File permissions** | Normalized to `0o755` / `0o644` by class |
| **Owner uid/gid** | Set to `0` |
| **Sort order** | Paths sorted lexicographically; CBOR map keys sorted by canonical encoding |
| **Floating-point metadata** | Forbidden entirely |
| **Compression** | Fixed zstd level (3), deterministic across runs |
| **File path separator** | Forward slash `/` always; never backslash |
| **Character encoding** | All paths and text strings MUST be valid UTF-8 |
| **Locale** | CBOR key sorting is bytewise, not locale-aware |
| **Endianness** | BLAKE3 output is deterministic; CBOR encoding is big-endian; base32 is bit-order defined by RFC 4648 |
| **Line endings** | Not normalized at the capsule level; canonical bytes include original bytes |
| **Hash function** | BLAKE3 is architecturally deterministic; the same input always produces the same digest |

### 7.1 Implementation requirements

Implementations MUST:

- Use the official BLAKE3 reference implementation or a verified port.
- Use RFC 4648 base32 (not Crockford base32, not base32hex).
- Use lowercase base32 output. Uppercase is a CID validation error.
- Strip padding (`=`) from base32 output.
- Enforce all canonicalization rules and reject non-canonical input.
- Use unsigned LEB128 varint encoding for the multihash header.
- Use big-endian byte order for all CBOR and multihash fields.
- Sort CBOR map keys by bytewise comparison of the canonical CBOR encoding of each key.
- Reject CBOR float types, indefinite-length encodings, and unknown tags.
- Validate UTF-8 well-formedness for all text strings in capsule metadata.

### 7.2 Conformance verification

The test vectors in `docs/schemas/test-vectors/cid/cid-vectors.json` provide a set of known canonical inputs and their expected CIDs. An implementation passes CID conformance when it produces the correct `expectedCid` for every vector.

---

*Back to the [specifications index](../specs/README.md).*
