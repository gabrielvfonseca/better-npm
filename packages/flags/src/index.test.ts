import { describe, it, expect } from "vitest"
import { FeatureFlags, createFeatureFlags } from "./index.js"

describe("@dash/flags", () => {
  it("should create a flags instance", () => {
    const flags = createFeatureFlags()
    expect(flags).toBeInstanceOf(FeatureFlags)
  })

  it("should disable feature by default", () => {
    const flags = new FeatureFlags()
    expect(flags.isEnabled("nonexistent")).toBe(false)
  })

  it("should enable and disable flags", () => {
    const flags = new FeatureFlags([{ name: "feature-x", enabled: true }])
    expect(flags.isEnabled("feature-x")).toBe(true)
    flags.setFlag("feature-x", false)
    expect(flags.isEnabled("feature-x")).toBe(false)
  })

  it("should return all flags", () => {
    const flags = new FeatureFlags([
      { name: "a", enabled: true },
      { name: "b", enabled: false },
    ])
    expect(flags.getAll()).toEqual({ a: true, b: false })
  })

  it("should evaluate rules with context", () => {
    const flags = new FeatureFlags([
      {
        name: "beta",
        enabled: true,
        rules: [{ field: "role", operator: "eq", value: "admin" }],
      },
    ])

    expect(flags.isEnabled("beta", { role: "admin" })).toBe(true)
    expect(flags.isEnabled("beta", { role: "user" })).toBe(false)
  })

  it("should add flags dynamically", () => {
    const flags = new FeatureFlags()
    flags.addFlag({ name: "new-feature", enabled: true })
    expect(flags.isEnabled("new-feature")).toBe(true)
  })
})
