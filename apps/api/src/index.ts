import { loadConfig } from "@better-npm/config"
import { createLogger } from "@better-npm/logger"
import { serve } from "@hono/node-server"
import { swaggerUI } from "@hono/swagger-ui"
import { Hono } from "hono"
import { cors } from "hono/cors"
import { HTTPException } from "hono/http-exception"
import { requestId } from "hono/request-id"

const logger = createLogger("api")
const config = loadConfig()

const app = new Hono()

app.use("*", cors({ origin: config.api?.corsOrigin ?? "*", credentials: true }))
app.use("*", requestId())
app.use("*", async (c, next) => {
  const start = performance.now()
  const reqId = c.get("requestId") as string | undefined
  logger.info(`${c.req.method} ${c.req.url}`, { requestId: reqId })
  await next()
  const ms = (performance.now() - start).toFixed(2)
  logger.info(`${c.res.status} ${c.req.method} ${c.req.url} ${ms}ms`, { requestId: reqId })
})

app.onError((err, c) => {
  if (err instanceof HTTPException) {
    return c.json({ error: err.message, status: err.status }, err.status)
  }
  logger.error("Unhandled error", { error: err.message, stack: err.stack })
  return c.json({ error: "Internal server error", status: 500 }, 500)
})

app.get("/health", (c) =>
  c.json({ status: "healthy", service: "@better-npm/api", timestamp: new Date().toISOString() }),
)

app.get("/docs", swaggerUI({ url: "/openapi.json" }))

app.get("/openapi.json", (c) => {
  return c.json({
    openapi: "3.1.0",
    info: { title: "@better-npm/api", version: "1.0.0", description: "Bnpm API Gateway" },
    servers: [{ url: config.api?.url ?? "http://localhost:3000" }],
    paths: {
      "/health": { get: { summary: "Health check", responses: { "200": { description: "OK" } } } },
    },
  })
})

const apiV1 = new Hono()
apiV1.get("/health", (c) => c.json({ status: "healthy", service: "@better-npm/api/v1" }))
app.route("/api/v1", apiV1)

export function createApp() {
  return app
}

export function start(port?: number) {
  const resolvedPort = port ?? config.api?.port ?? 3000

  const server = serve({ fetch: app.fetch, port: resolvedPort }, (info) => {
    logger.info(`API server listening on http://localhost:${info.port}`)
  })

  const shutdown = async () => {
    logger.info("Shutting down API server...")
    server.close()
    process.exit(0)
  }

  process.on("SIGTERM", shutdown)
  process.on("SIGINT", shutdown)

  return server
}
