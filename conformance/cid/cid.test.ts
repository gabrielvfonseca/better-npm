import { describe, expect, it } from "vitest"
import { runConformance } from "../helpers/runner"

describe("CID construction", () => {
  const report = runConformance("cid")

  it("produces correct CID from canonical bytes", () => {
    const vectorCases = report.cases.filter((c) => c.name.startsWith("vector["))
    expect(vectorCases.length).toBeGreaterThanOrEqual(12)
    for (const c of vectorCases) {
      expect(c.ok, `${c.name}: expected=${c.expected} got=${c.got}`).toBe(true)
    }
  })

  it("rejects non-canonical input", () => {
    const c = report.cases.find((c) => c.name.includes("non-canonical"))
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })

  it("base32 encoding matches spec", () => {
    const vectorCases = report.cases.filter((c) => c.name.startsWith("vector["))
    for (const c of vectorCases) {
      expect(c.ok, `CID base32 mismatch: ${c.name}`).toBe(true)
    }
  })

  it("deterministic across platforms", () => {
    const c = report.cases.find((c) => c.name.includes("deterministic"))
    expect(c).toBeDefined()
    expect(c?.ok).toBe(true)
  })
})
