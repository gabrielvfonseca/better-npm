import { describe, expect, it, vi } from "vitest"
import { AnalyticsClient, createAnalyticsClient } from "./index.js"

describe("@better-npm/analytics", () => {
  it("should create a client", () => {
    const client = createAnalyticsClient({ enabled: false })
    expect(client).toBeInstanceOf(AnalyticsClient)
  })

  it("should queue events when disabled", () => {
    const client = new AnalyticsClient({ enabled: false })
    expect(() => {
      client.track({
        name: "test",
        timestamp: new Date().toISOString(),
        sessionId: "test",
      })
    }).not.toThrow()
  })

  it("should track page views", () => {
    const client = new AnalyticsClient({ enabled: false })
    expect(() => client.trackPageView("/home")).not.toThrow()
  })

  it("should identify users", () => {
    const client = new AnalyticsClient({ enabled: false })
    expect(() => client.identify("user-1", { plan: "premium" })).not.toThrow()
  })

  it("should flush without error when disabled", async () => {
    const client = new AnalyticsClient({ enabled: false })
    await expect(client.flush()).resolves.toBeUndefined()
  })

  it("should attempt to send events on flush", async () => {
    globalThis.fetch = vi.fn().mockResolvedValue({ ok: true }) as unknown as typeof fetch
    const client = new AnalyticsClient({ enabled: true, endpoint: "http://localhost:9999" })
    client.track({ name: "test", timestamp: new Date().toISOString(), sessionId: "sess" })
    await client.flush()
    expect(fetch).toHaveBeenCalled()
  })
})
