import { loadConfig } from "@better-npm/config"
import { createLogger } from "@better-npm/logger"
import { type TelemetryClient, createTelemetryClient } from "@better-npm/telemetry"
import { Hono } from "hono"
import { HTTPException } from "hono/http-exception"
import type { ContentfulStatusCode } from "hono/utils/http-status"

const logger = createLogger("status")
const config = loadConfig()

const app = new Hono()
let telemetry: TelemetryClient | undefined = undefined

interface ComponentStatus {
  name: string
  status: "operational" | "degraded" | "down" | "maintenance"
  latency?: number
  lastChecked: string
  message?: string
}

app.use("*", async (c, next) => {
  const start = performance.now()
  await next()
  const ms = (performance.now() - start).toFixed(2)
  logger.info(`${c.res.status} ${c.req.method} ${c.req.path} ${ms}ms`)
})

app.get("/status", async (c) => {
  try {
    const components = await checkAllComponents()
    const overallStatus = determineOverallStatus(components)

    return c.json({
      status: overallStatus,
      service: "@better-npm/status",
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
      components,
    })
  } catch (err) {
    logger.error("Failed to get system status", { error: String(err) })
    throw new HTTPException(500, { message: "Failed to retrieve system status" })
  }
})

app.get("/status/components", async (c) => {
  try {
    const components = await checkAllComponents()
    return c.json({ components, timestamp: new Date().toISOString() })
  } catch (err) {
    logger.error("Failed to get component statuses", { error: String(err) })
    throw new HTTPException(500, { message: "Failed to retrieve component statuses" })
  }
})

app.get("/status/incidents", async (c) => {
  try {
    const page = Math.max(1, Number.parseInt(c.req.query("page") ?? "1", 10))
    const limit = Math.min(100, Math.max(1, Number.parseInt(c.req.query("limit") ?? "20", 10)))

    const incidents = await getIncidentHistory(page, limit)
    return c.json({
      incidents: incidents.data,
      pagination: {
        page,
        limit,
        total: incidents.total,
        totalPages: Math.ceil(incidents.total / limit),
      },
      timestamp: new Date().toISOString(),
    })
  } catch (err) {
    logger.error("Failed to get incident history", { error: String(err) })
    throw new HTTPException(500, { message: "Failed to retrieve incident history" })
  }
})

app.get("/health", (c) => {
  return c.json({
    status: "healthy",
    service: "@better-npm/status",
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
  })
})

app.onError((err, c) => {
  if (err instanceof HTTPException) {
    return c.json({ error: err.message, status: err.status }, err.status as ContentfulStatusCode)
  }
  logger.error("Unhandled error", { error: err.message })
  return c.json({ error: "Internal server error", status: 500 }, 500)
})

app.notFound((c) => c.json({ error: "Not found", status: 404 }, 404))

async function checkAllComponents(): Promise<ComponentStatus[]> {
  const checks: ComponentStatus[] = []

  checks.push(await checkComponent("API", "http://localhost:3000/health"))
  checks.push(await checkComponent("Auth", "http://localhost:3001/health"))
  checks.push(await checkComponent("Registry", "http://localhost:3002/health"))
  checks.push(await checkComponent("Search", "http://localhost:3003/health"))
  checks.push(await checkComponent("Telemetry", "http://localhost:3004/health"))

  if (telemetry) {
    try {
      const dbHealth = await telemetry.checkDatabase()
      checks.push({
        name: "Database",
        status: dbHealth.connected ? "operational" : "down",
        lastChecked: new Date().toISOString(),
        message: dbHealth.connected ? undefined : "Database connection failed",
      })
    } catch {
      checks.push({
        name: "Database",
        status: "down",
        lastChecked: new Date().toISOString(),
        message: "Health check failed",
      })
    }
  }

  return checks
}

async function checkComponent(name: string, healthUrl: string): Promise<ComponentStatus> {
  const start = performance.now()
  try {
    const response = await fetch(healthUrl, { signal: AbortSignal.timeout(5000) })
    const latency = performance.now() - start

    if (response.ok) {
      return { name, status: "operational", latency, lastChecked: new Date().toISOString() }
    }

    return {
      name,
      status: "degraded",
      latency,
      lastChecked: new Date().toISOString(),
      message: `HTTP ${response.status}`,
    }
  } catch {
    const latency = performance.now() - start
    return {
      name,
      status: "down",
      latency,
      lastChecked: new Date().toISOString(),
      message: "Unreachable",
    }
  }
}

function determineOverallStatus(components: ComponentStatus[]): string {
  if (components.some((c) => c.status === "down")) return "down"
  if (components.some((c) => c.status === "degraded")) return "degraded"
  if (components.some((c) => c.status === "maintenance")) return "maintenance"
  return "operational"
}

async function getIncidentHistory(
  _page: number,
  _limit: number,
): Promise<{ data: unknown[]; total: number }> {
  return { data: [], total: 0 }
}

export async function createStatusService(telemetryClient?: TelemetryClient) {
  telemetry = telemetryClient ?? (config.telemetry ? await createTelemetryClient() : undefined)
  logger.info("Status service initialized")
  return app
}

export default app
