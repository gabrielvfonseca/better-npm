import { describe, expect, it } from "vitest"
import { runConformance } from "../helpers/runner"

describe("Store", () => {
  const report = runConformance("store")

  it("stores and retrieves by CID", () => {
    const c = report.cases.find((c) => c.name === "store_and_retrieve_by_cid")
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })

  it("file-level dedup across packages", () => {
    const c = report.cases.find((c) => c.name === "file_level_dedup_across_capsules")
    expect(c).toBeDefined()
    expect(c?.ok, `expected ${c?.expected}, got ${c?.got}`).toBe(true)
  })

  it("materialization: reflink→hardlink→copy fallback", () => {
    const c = report.cases.find((c) => c.name === "materialize_and_verify_files")
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })

  it("local verify detects tampered bytes", () => {
    const c = report.cases.find((c) => c.name === "tamper_detection_verify_fails")
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })
})
