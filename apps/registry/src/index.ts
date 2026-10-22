import { type AuthToken, parseAuthHeader, verifyToken } from "@better-npm/auth"
import { loadConfig } from "@better-npm/config"
import { createLogger } from "@better-npm/logger"
import { type PackageStorage, PackageStorageFactory, PackageStoreError } from "@better-npm/storage"
import { publishPackageSchema, validate } from "@better-npm/validation"
import { type Context, Hono, type Next } from "hono"
import { HTTPException } from "hono/http-exception"
import { z } from "zod"

type Env = { Variables: { user: AuthToken } }

const logger = createLogger("registry")
const config = loadConfig()

const app = new Hono<Env>()
let packageStorage: PackageStorage = PackageStorageFactory.create()

const PUBLISHER_ROLES = ["publisher", "admin"]

const requirePublisher = async (c: Context<Env>, next: Next) => {
  const header = parseAuthHeader(c.req.header("authorization") ?? "")
  if (!header.ok) throw new HTTPException(401, { message: "Authentication required" })

  const token = await verifyToken(header.value, config.auth.secret)
  if (!token.ok) throw new HTTPException(401, { message: "Invalid or expired token" })

  if (!token.value.roles.some((role) => PUBLISHER_ROLES.includes(role))) {
    throw new HTTPException(403, { message: "Publisher role required" })
  }

  c.set("user", token.value)
  await next()
}

const assertCanModify = async (c: Context<Env>, name: string): Promise<void> => {
  const pkg = await packageStorage.getPackage(name)
  if (!pkg) return

  const user = c.get("user")
  if (user.roles.includes("admin") || pkg.owners.includes(user.sub)) return
  throw new HTTPException(403, { message: `You are not an owner of '${name}'` })
}

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

    const storage = packageStorage
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

    const storage = packageStorage
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

app.put("/packages/:name", requirePublisher, async (c) => {
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

    await assertCanModify(c, name)
    const result = await packageStorage.publishPackage(name, payload, c.get("user").sub)
    logger.info("Package published", { name, version: payload.version })

    return c.json({ package: result }, 201)
  } catch (err) {
    if (err instanceof HTTPException) throw err
    if (err instanceof PackageStoreError)
      throw new HTTPException(err.status, { message: err.message })
    logger.error("Failed to publish package", { name: c.req.param("name"), error: String(err) })
    throw new HTTPException(500, { message: "Failed to publish package" })
  }
})

app.delete("/packages/:name/:version", requirePublisher, async (c) => {
  try {
    const name = c.req.param("name")
    const version = c.req.param("version")
    if (!(name && version))
      throw new HTTPException(400, { message: "Package name and version are required" })

    await assertCanModify(c, name)
    const removed = await packageStorage.unpublishPackageVersion(name, version)
    if (!removed)
      throw new HTTPException(404, { message: `Package '${name}@${version}' not found` })
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

    const storage = packageStorage
    const results = await storage.searchPackages(query)
    return c.json({ results, query, total: results.length })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Search failed", { query: c.req.query("q"), error: String(err) })
    throw new HTTPException(500, { message: "Search failed" })
  }
})

app.post("/packages/:name/deprecate", requirePublisher, async (c) => {
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

    await assertCanModify(c, name)
    const storage = packageStorage
    const result = version
      ? await storage.deprecatePackageVersion(name, version, reason)
      : await storage.deprecatePackage(name, reason)
    if (!result) {
      throw new HTTPException(404, {
        message: version ? `Package '${name}@${version}' not found` : `Package '${name}' not found`,
      })
    }

    logger.info("Package deprecated", { name, version, reason })
    return c.json({ package: result, deprecated: true, reason })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Failed to deprecate package", { name: c.req.param("name"), error: String(err) })
    throw new HTTPException(500, { message: "Failed to deprecate package" })
  }
})

app.get("/health", (c) => c.json({ status: "healthy", service: "@better-npm/registry" }))

app.onError((err, c) => {
  if (err instanceof HTTPException) {
    return c.json({ error: err.message, status: err.status }, err.status)
  }
  logger.error("Unhandled error", { error: err.message })
  return c.json({ error: "Internal server error", status: 500 }, 500)
})

app.notFound((c) => c.json({ error: "Not found", status: 404 }, 404))

export async function createRegistry(storageBackend?: PackageStorage) {
  packageStorage = storageBackend ?? PackageStorageFactory.create()
  logger.info("Registry initialized with storage backend")
  return app
}

export default app
