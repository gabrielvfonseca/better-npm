import { describe, it, expect, afterEach } from "vitest"
import { Dash, createDash, loadConfig } from "./index.js"

describe("@dash/sdk", () => {
  const originalEnv = { ...process.env }

  afterEach(() => {
    process.env = { ...originalEnv }
  })

  it("should create a Dash instance", () => {
    const dash = createDash({ baseUrl: "https://api.dash.dev" })
    expect(dash).toBeInstanceOf(Dash)
  })

  it("should initialize client", () => {
    const dash = new Dash({ baseUrl: "https://api.dash.dev" })
    expect(dash.client).toBeDefined()
  })

  it("should initialize logger", () => {
    const dash = new Dash()
    expect(dash.logger).toBeDefined()
  })

  it("should expose schemas", () => {
    const dash = new Dash()
    expect(dash.schemas.packageName).toBeDefined()
    expect(dash.schemas.login).toBeDefined()
  })

  it("should expose auth utilities", () => {
    const dash = new Dash()
    expect(dash.auth.verifyToken).toBeDefined()
    expect(dash.auth.createToken).toBeDefined()
  })

  it("should expose env utilities", () => {
    const dash = new Dash()
    expect(typeof dash.env.isDev).toBe("function")
    expect(typeof dash.env.isProd).toBe("function")
  })

  it("should load config", () => {
    expect(loadConfig).toBeDefined()
  })
})
