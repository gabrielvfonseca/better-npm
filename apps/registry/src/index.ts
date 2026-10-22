import { type AuthToken, parseAuthHeader, verifyToken } from "@better-npm/auth"
import { loadConfig } from "@better-npm/config"
import { createLogger } from "@better-npm/logger"
import { type PackageStorage, PackageStorageFactory, PackageStoreError } from "@better-npm/storage"
import {
  packageNameSchema,
  publishPackageSchema,
  semVerSchema,
  validate,
} from "@better-npm/validation"
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

const parsePackageName = (raw: string | undefined): string => {
  const result = validate(packageNameSchema, raw)
  if (!result.ok) throw new HTTPException(400, { message: "Invalid package name" })
  return result.value
}

const parseVersion = (raw: string | undefined): string => {
  const result = validate(semVerSchema, raw)
  if (!result.ok) throw new HTTPException(400, { message: "Invalid version" })
  return result.value
}

const actorOf = (c: Context<Env>) => {
  const user = c.get("user")
  return { id: user.sub, admin: user.roles.includes("admin") }
}

const toHttpException = (err: unknown): HTTPException | undefined => {
  if (err instanceof HTTPException) return err
  if (err instanceof PackageStoreError)
    return new HTTPException(err.status, { message: err.message })
  return undefined
}

app.use("*", async (c, next) => {
  const start = performance.now()
  await next()
  const ms = (performance.now() - start).toFixed(2)
  logger.info(`${c.res.status} ${c.req.method} ${c.req.path} ${ms}ms`)
})

app.get("/packages/:name", async (c) => {
  try {
    const name = parsePackageName(c.req.param("name"))

    const pkg = await packageStorage.getPackage(name)
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
    const name = parsePackageName(c.req.param("name"))
    const version = parseVersion(c.req.param("version"))

    const pkg = await packageStorage.getPackageVersion(name, version)
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
    const name = parsePackageName(c.req.param("name"))

    const body = await c.req.json()
    const validationResult = validate(publishPackageSchema, body)
    if (!validationResult.ok) {
      throw new HTTPException(400, {
        message: "Invalid package data",
        cause: validationResult.error,
      })
    }
    const payload = validationResult.value
    if (payload.name !== name) {
      throw new HTTPException(400, { message: "Package name in body does not match the URL" })
    }

    const result = await packageStorage.publishPackage(name, payload, actorOf(c))
    logger.info("Package published", { name, version: payload.version })

    return c.json({ package: result }, 201)
  } catch (err) {
    const httpErr = toHttpException(err)
    if (httpErr) throw httpErr
    logger.error("Failed to publish package", { name: c.req.param("name"), error: String(err) })
    throw new HTTPException(500, { message: "Failed to publish package" })
  }
})

app.delete("/packages/:name/:version", requirePublisher, async (c) => {
  try {
    const name = parsePackageName(c.req.param("name"))
    const version = parseVersion(c.req.param("version"))

    const removed = await packageStorage.unpublishPackageVersion(name, version, actorOf(c))
    if (!removed)
      throw new HTTPException(404, { message: `Package '${name}@${version}' not found` })
    logger.info("Package version unpublished", { name, version })

    return c.json({ message: `Package '${name}@${version}' unpublished` })
  } catch (err) {
    const httpErr = toHttpException(err)
    if (httpErr) throw httpErr
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

    const results = await packageStorage.searchPackages(query)
    return c.json({ results, query, total: results.length })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Search failed", { query: c.req.query("q"), error: String(err) })
    throw new HTTPException(500, { message: "Search failed" })
  }
})

app.post("/packages/:name/deprecate", requirePublisher, async (c) => {
  try {
    const name = parsePackageName(c.req.param("name"))
    const versionParam = c.req.query("version")
    const version = versionParam === undefined ? undefined : parseVersion(versionParam)

    const body = await c.req.json()
    const validationResult = validate(z.object({ reason: z.string() }), body)
    if (!validationResult.ok) {
      throw new HTTPException(400, {
        message: "Invalid deprecation data",
        cause: validationResult.error,
      })
    }
    const { reason } = validationResult.value

    const result = version
      ? await packageStorage.deprecatePackageVersion(name, version, reason, actorOf(c))
      : await packageStorage.deprecatePackage(name, reason, actorOf(c))
    if (!result) {
      throw new HTTPException(404, {
        message: version ? `Package '${name}@${version}' not found` : `Package '${name}' not found`,
      })
    }

    logger.info("Package deprecated", { name, version, reason })
    return c.json({ package: result, deprecated: true, reason })
  } catch (err) {
    const httpErr = toHttpException(err)
    if (httpErr) throw httpErr
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
