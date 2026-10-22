import { describe, expect, it } from "vitest"
import {
  createTestConfig,
  createTestLogger,
  createTestPackage,
  randomId,
  withTempDir,
} from "./index.js"

describe("@better-npm/testing", () => {
  it("should create a noop logger", () => {
    const logger = createTestLogger()
    expect(logger).toBeDefined()
    expect(() => logger.debug("test")).not.toThrow()
    expect(() => logger.child("child")).not.toThrow()
  })

  it("should create a test config", () => {
    const config = createTestConfig()
    expect(config.api.url).toBe("http://localhost:3000")
    expect(config.telemetry.enabled).toBe(false)
  })

  it("should create test config with overrides", () => {
    const config = createTestConfig({
      api: { url: "http://custom:8080", port: 8080, host: "0.0.0.0", corsOrigin: "*" },
    })
    expect(config.api.url).toBe("http://custom:8080")
  })

  it("should generate random IDs", () => {
    const id = randomId()
    expect(id).toContain("test_")
  })

  it("should create test packages", () => {
    const pkg = createTestPackage()
    expect(pkg.name).toContain("@better-npm/test-")
    expect(pkg.version).toBe("1.0.0")
    expect(pkg.main).toBe("./src/index.js")
  })

  it("should create test packages with overrides", () => {
    const pkg = createTestPackage({ version: "2.0.0", private: true })
    expect(pkg.version).toBe("2.0.0")
    expect(pkg.private).toBe(true)
  })

  it("should work with temp directories", async () => {
    let _dirPath = ""
    await withTempDir(async (dir) => {
      _dirPath = dir
      expect(dir).toBeTruthy()
    })
    // dir should be cleaned up after
  })
})
