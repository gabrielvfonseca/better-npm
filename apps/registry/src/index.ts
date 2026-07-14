import { loadConfig } from "@dash/config"
import { createLogger } from "@dash/logger"
import { StorageFactory } from "@dash/storage"
import { publishPackageSchema, validate } from "@dash/validation"
import { Hono } from "hono"
import { HTTPException } from "hono/http-exception"
import { z } from "zod"

const logger = createLogger("registry")
const _config = loadConfig()

const app = new Hono()
let _storage: ReturnType<typeof StorageFactory.create>

app.use("*", async (c, next) => {
  const start = performance.now()
  await next()
  const ms = (performance.now() - start).toFixed(2)
  logger.info(`${c.res.status} ${c.req.method} ${c.req.path} ${ms}ms`)
})

app.get("/packages/:name", async (c) => {
  try {
    const name = c.req.param("name")
    if (!name) throw new HTTPException(400, { message: "Package name is required" })

    const storage = StorageFactory.create({ type: "local" })
    const pkg = await storage.getPackage(name)
    if (!pkg) throw new HTTPException(404, { message: `Package '${name}' not found` })

    return c.json({ package: pkg })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Failed to get package", { name: c.req.param("name"), error: String(err) })
    throw new HTTPException(500, { message: "Failed to retrieve package" })
  }
})

app.get("/packages/:name/:version", async (c) => {
  try {
    const name = c.req.param("name")
    const version = c.req.param("version")
    if (!(name && version))
      throw new HTTPException(400, { message: "Package name and version are required" })

    const storage = StorageFactory.create({ type: "local" })
    const pkg = await storage.getPackageVersion(name, version)
    if (!pkg) throw new HTTPException(404, { message: `Package '${name}@${version}' not found` })

    return c.json({ package: pkg })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Failed to get package version", {
      name: c.req.param("name"),
      version: c.req.param("version"),
      error: String(err),
    })
    throw new HTTPException(500, { message: "Failed to retrieve package version" })
  }
})

app.put("/packages/:name", async (c) => {
  try {
    const name = c.req.param("name")
    if (!name) throw new HTTPException(400, { message: "Package name is required" })

    const body = await c.req.json()
    const validationResult = validate(publishPackageSchema, body)
    if (!validationResult.ok) {
      throw new HTTPException(400, {
        message: "Invalid package data",
        cause: validationResult.error,
      })
    }
    const payload = validationResult.value

    const storage = StorageFactory.create({ type: "local" })
    const result = await storage.publishPackage(name, payload)
    logger.info("Package published", { name, version: payload.version })

    return c.json({ package: result }, 201)
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Failed to publish package", { name: c.req.param("name"), error: String(err) })
    throw new HTTPException(500, { message: "Failed to publish package" })
  }
})

app.delete("/packages/:name/:version", async (c) => {
  try {
    const name = c.req.param("name")
    const version = c.req.param("version")
    if (!(name && version))
      throw new HTTPException(400, { message: "Package name and version are required" })

    const storage = StorageFactory.create({ type: "local" })
    await storage.unpublishPackageVersion(name, version)
    logger.info("Package version unpublished", { name, version })

    return c.json({ message: `Package '${name}@${version}' unpublished` })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Failed to unpublish package version", {
      name: c.req.param("name"),
      version: c.req.param("version"),
      error: String(err),
    })
    throw new HTTPException(500, { message: "Failed to unpublish package version" })
  }
})

app.get("/search", async (c) => {
  try {
    const query = c.req.query("q")
    if (!query) throw new HTTPException(400, { message: "Search query is required" })

    const storage = StorageFactory.create({ type: "local" })
    const results = await storage.searchPackages(query)
    return c.json({ results, query, total: results.length })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Search failed", { query: c.req.query("q"), error: String(err) })
    throw new HTTPException(500, { message: "Search failed" })
  }
})

app.post("/packages/:name/deprecate", async (c) => {
  try {
    const name = c.req.param("name")
    const version = c.req.query("version")
    if (!name) throw new HTTPException(400, { message: "Package name is required" })

    const body = await c.req.json()
    const validationResult = validate(z.object({ reason: z.string() }), body)
    if (!validationResult.ok) {
      throw new HTTPException(400, {
        message: "Invalid deprecation data",
        cause: validationResult.error,
      })
    }
    const { reason } = validationResult.value

    const storage = StorageFactory.create({ type: "local" })
    const result = version
      ? await storage.deprecatePackageVersion(name, version, reason)
      : await storage.deprecatePackage(name, reason)

    logger.info("Package deprecated", { name, version, reason })
    return c.json({ package: result, deprecated: true, reason })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Failed to deprecate package", { name: c.req.param("name"), error: String(err) })
    throw new HTTPException(500, { message: "Failed to deprecate package" })
  }
})

app.get("/health", (c) => c.json({ status: "healthy", service: "@dash/registry" }))

app.onError((err, c) => {
  if (err instanceof HTTPException) {
    return c.json({ error: err.message, status: err.status }, err.status)
  }
  logger.error("Unhandled error", { error: err.message })
  return c.json({ error: "Internal server error", status: 500 }, 500)
})

app.notFound((c) => c.json({ error: "Not found", status: 404 }, 404))

export async function createRegistry(storageBackend?: ReturnType<typeof StorageFactory.create>) {
  const _storage = storageBackend ?? StorageFactory.create({ type: "local" })
  logger.info("Registry initialized with storage backend")
  return app
}

export default app
