import { afterEach, describe, expect, it } from "vitest"
import {
  getTracer,
  incrementCounter,
  recordHistogram,
  recordMetric,
  setAttribute,
  shutdownTracing,
  startTracing,
  wrapWithSpan,
} from "./index.js"

describe("@dash/telemetry", () => {
  afterEach(async () => {
    await shutdownTracing()
  })

  it("should start tracing without crashing", () => {
    startTracing({ serviceName: "test" })
    expect(getTracer()).toBeDefined()
  })

  it("should support shutdown", async () => {
    startTracing({ serviceName: "test" })
    await shutdownTracing()
    await shutdownTracing() // double shutdown should not throw
  })

  it("should record metrics without errors", () => {
    recordMetric("test.metric", 42, { env: "test" })
    incrementCounter("test.counter")
    incrementCounter("test.counter", 5)
    recordHistogram("test.histogram", 100)
  })

  it("should set attributes", () => {
    setAttribute("key", "value")
  })

  it("should get tracer", () => {
    const t = getTracer()
    const span = t.startSpan("test")
    expect(span).toBeDefined()
    span.end()
  })

  it("should wrap async functions with span", async () => {
    const result = await wrapWithSpan("test", async () => "done")
    expect(result).toBe("done")
  })

  it("should handle errors in wrapped functions", async () => {
    await expect(
      wrapWithSpan("failing", async () => {
        throw new Error("fail")
      }),
    ).rejects.toThrow("fail")
  })
})
