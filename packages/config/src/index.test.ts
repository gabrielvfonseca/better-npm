import { describe, it, expect, afterEach } from "vitest"
import { getConfig, loadConfig, getEnvVar, isDevelopment, isProduction } from "./index.js"

describe("@dash/config", () => {
  const originalEnv = { ...process.env }

  afterEach(() => {
    process.env = { ...originalEnv }
  })

  it("should load default config", () => {
    const config = loadConfig()
    expect(config.api.url).toBe("http://localhost:3000")
    expect(config.api.port).toBe(3000)
    expect(config.registry.storagePath).toBe("./data/registry")
  })

  it("should read from environment variables", () => {
    process.env["API_URL"] = "https://api.example.com"
    process.env["API_PORT"] = "8080"
    const config = loadConfig()
    expect(config.api.url).toBe("https://api.example.com")
    expect(config.api.port).toBe(8080)
  })

  it("should return cached config", () => {
    const a = loadConfig()
    const b = getConfig()
    expect(a).toBe(b)
  })

  it("should get env var with default", () => {
    expect(getEnvVar("NONEXISTENT_VAR_XYZ", "default")).toBe("default")
  })

  it("should detect development environment", () => {
    process.env["NODE_ENV"] = "development"
    expect(isDevelopment()).toBe(true)
  })

  it("should detect production environment", () => {
    process.env["NODE_ENV"] = "production"
    expect(isProduction()).toBe(true)
    expect(isDevelopment()).toBe(false)
  })
})
