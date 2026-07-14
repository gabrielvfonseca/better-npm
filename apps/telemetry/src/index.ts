import { loadConfig } from "@dash/config"
import { createLogger } from "@dash/logger"
import { type TelemetryClient, createTelemetryClient } from "@dash/telemetry"
import { Hono } from "hono"
import { HTTPException } from "hono/http-exception"
import type { ContentfulStatusCode } from "hono/utils/http-status"

const logger = createLogger("telemetry")
const config = loadConfig()

const app = new Hono()
let telemetry: TelemetryClient

app.use("*", async (c, next) => {
  const start = performance.now()
  await next()
  const ms = (performance.now() - start).toFixed(2)
  logger.info(`${c.res.status} ${c.req.method} ${c.req.path} ${ms}ms`)
})

app.post("/events", async (c) => {
  try {
    const body = await c.req.json()
    const events = Array.isArray(body) ? body : [body]

    if (events.length === 0) {
      throw new HTTPException(400, { message: "At least one event is required" })
    }

    if (events.length > 1000) {
      throw new HTTPException(413, { message: "Batch size exceeds maximum of 1000 events" })
    }

    for (const event of events) {
      if (!(event.name && event.timestamp)) {
        throw new HTTPException(400, {
          message: "Each event must have 'name' and 'timestamp' fields",
        })
      }
    }

    const results = await telemetry.ingestEvents(events)
    const accepted = results.filter((r: { accepted: boolean }) => r.accepted).length
    const rejected = results.length - accepted

    logger.info("Events ingested", { total: events.length, accepted, rejected })
    return c.json({ accepted, rejected, events: results }, 201)
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Event ingestion failed", { error: String(err) })
    throw new HTTPException(500, { message: "Failed to ingest events" })
  }
})

app.post("/metrics", async (c) => {
  try {
    const body = await c.req.json()
    const metrics = Array.isArray(body) ? body : [body]

    if (metrics.length === 0) {
      throw new HTTPException(400, { message: "At least one metric is required" })
    }

    if (metrics.length > 5000) {
      throw new HTTPException(413, { message: "Batch size exceeds maximum of 5000 metrics" })
    }

    for (const metric of metrics) {
      if (!metric.name || metric.value === undefined) {
        throw new HTTPException(400, { message: "Each metric must have 'name' and 'value' fields" })
      }
    }

    const results = await telemetry.ingestMetrics(metrics)
    const accepted = results.filter((r: { accepted: boolean }) => r.accepted).length
    const rejected = results.length - accepted

    logger.info("Metrics ingested", { total: metrics.length, accepted, rejected })
    return c.json({ accepted, rejected, metrics: results }, 201)
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Metric ingestion failed", { error: String(err) })
    throw new HTTPException(500, { message: "Failed to ingest metrics" })
  }
})

app.get("/health", (c) => {
  const connected = telemetry?.isConnected() ?? false
  return c.json({
    status: connected ? "healthy" : "degraded",
    service: "@dash/telemetry",
    client: connected ? "connected" : "disconnected",
  })
})

app.get("/metrics", async (c) => {
  try {
    const prometheusMetrics = await telemetry.exportPrometheus()
    c.header("Content-Type", "text/plain; charset=utf-8")
    return c.body(prometheusMetrics)
  } catch (err) {
    logger.error("Failed to export Prometheus metrics", { error: String(err) })
    throw new HTTPException(500, { message: "Failed to export metrics" })
  }
})

app.onError((err, c) => {
  if (err instanceof HTTPException) {
    return c.json({ error: err.message }, { status: err.status as ContentfulStatusCode })
  }
  logger.error("Unhandled error", { error: err.message })
  return c.json({ error: "Internal server error" }, { status: 500 })
})

app.notFound((c) => c.json({ error: "Not found", status: 404 }, 404))

export async function createTelemetryService(client?: TelemetryClient) {
  telemetry = client ?? (await createTelemetryClient(config.telemetry))
  logger.info("Telemetry service initialized")
  return app
}

export default app
