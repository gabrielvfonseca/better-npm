import { afterEach, describe, expect, it } from "vitest"
import { Bnpm, createBnpm, loadConfig } from "./index.js"

describe("@better-npm/sdk", () => {
  const originalEnv = { ...process.env }

  afterEach(() => {
    process.env = { ...originalEnv }
  })

  it("should create a Bnpm instance", () => {
    const bnpm = createBnpm({ baseUrl: "https://api.bnpm.dev" })
    expect(bnpm).toBeInstanceOf(Bnpm)
  })

  it("should initialize client", () => {
    const bnpm = new Bnpm({ baseUrl: "https://api.bnpm.dev" })
    expect(bnpm.client).toBeDefined()
  })

  it("should initialize logger", () => {
    const bnpm = new Bnpm()
    expect(bnpm.logger).toBeDefined()
  })

  it("should expose schemas", () => {
    const bnpm = new Bnpm()
    expect(bnpm.schemas.packageName).toBeDefined()
    expect(bnpm.schemas.login).toBeDefined()
  })

  it("should expose auth utilities", () => {
    const bnpm = new Bnpm()
    expect(bnpm.auth.verifyToken).toBeDefined()
    expect(bnpm.auth.createToken).toBeDefined()
  })

  it("should expose env utilities", () => {
    const bnpm = new Bnpm()
    expect(typeof bnpm.env.isDev).toBe("function")
    expect(typeof bnpm.env.isProd).toBe("function")
  })

  it("should load config", () => {
    expect(loadConfig).toBeDefined()
  })
})
