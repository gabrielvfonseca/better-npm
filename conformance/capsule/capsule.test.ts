import { describe, expect, it } from "vitest"
import { runConformance } from "../helpers/runner"

describe("Capsule format", () => {
  const report = runConformance("capsule")

  it("packs and unpacks correctly", () => {
    const c = report.cases.find((c) => c.name === "pack_and_unpack_roundtrip")
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })

  it("deterministic: same input → same CID", () => {
    const c = report.cases.find((c) => c.name === "deterministic_same_cid")
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })

  it("file-level dedup: identical files produce same hash", () => {
    const c = report.cases.find((c) => c.name === "file_level_dedup_same_hash")
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })

  it("range fetch returns correct byte ranges", () => {
    const c = report.cases.find((c) => c.name === "range_read")
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })

  it("detached attestations don't change content CID", () => {
    const c = report.cases.find((c) => c.name === "detached_attestation_cid_unchanged")
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })
})
