import { describe, it, expect } from "vitest"
import { ICON_NAMES } from "./index.js"

describe("@dash/icons", () => {
  it("should export icon names", () => {
    expect(ICON_NAMES).toContain("search")
    expect(ICON_NAMES).toContain("chevron-down")
    expect(ICON_NAMES).toContain("github")
    expect(ICON_NAMES).toContain("npm")
    expect(ICON_NAMES).toHaveLength(47)
  })

  it("should have no duplicates", () => {
    expect(new Set(ICON_NAMES).size).toBe(ICON_NAMES.length)
  })
})
