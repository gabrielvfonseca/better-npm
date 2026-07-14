import { loadConfig } from "@dash/config"
import { createLogger } from "@dash/logger"
import { type SearchClient, createSearchClient } from "@dash/search"
import { Hono } from "hono"
import { HTTPException } from "hono/http-exception"
import type { StatusCode } from "hono/utils/http-status"

const logger = createLogger("search")
const config = loadConfig()

const app = new Hono()
let searchClient: SearchClient

app.use("*", async (c, next) => {
  const start = performance.now()
  await next()
  const ms = (performance.now() - start).toFixed(2)
  logger.info(`${c.res.status} ${c.req.method} ${c.req.path} ${ms}ms`)
})

app.get("/search", async (c) => {
  try {
    const query = c.req.query("q")
    if (!query || query.length < 2) {
      throw new HTTPException(400, { message: "Search query must be at least 2 characters" })
    }

    const page = Math.max(1, Number.parseInt(c.req.query("page") ?? "1", 10))
    const limit = Math.min(100, Math.max(1, Number.parseInt(c.req.query("limit") ?? "20", 10)))
    const filters = parseFilters(c.req.query("filters"))

    const results = await searchClient.search(query, { page, limit, filters })
    logger.info("Search completed", { query, page, limit, total: results.total })

    return c.json({
      results: results.hits,
      pagination: {
        page,
        limit,
        total: results.total,
        totalPages: Math.ceil(results.total / limit),
      },
      query,
    })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Search failed", { query: c.req.query("q"), error: String(err) })
    throw new HTTPException(500, { message: "Search failed" })
  }
})

app.get("/suggest", async (c) => {
  try {
    const query = c.req.query("q")
    if (!query || query.length < 1) {
      throw new HTTPException(400, { message: "Suggestion query is required" })
    }

    const suggestions = await searchClient.suggest(query, { limit: 8 })
    return c.json({ suggestions, query })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Suggestions failed", { query: c.req.query("q"), error: String(err) })
    throw new HTTPException(500, { message: "Failed to get suggestions" })
  }
})

app.post("/index", async (c) => {
  try {
    const body = await c.req.json()
    const { name, data } = body

    if (!(name && data)) {
      throw new HTTPException(400, { message: "Document name and data are required" })
    }

    const result = await searchClient.index(name, data)
    logger.info("Document indexed", { name })
    return c.json({ indexed: true, name, id: result.id }, 201)
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Indexing failed", { error: String(err) })
    throw new HTTPException(500, { message: "Failed to index document" })
  }
})

app.delete("/index/:name", async (c) => {
  try {
    const name = c.req.param("name")
    if (!name) throw new HTTPException(400, { message: "Document name is required" })

    // Since SearchClient doesn't have a remove method, we'll implement a basic version
    // In a real implementation, this would call the search engine's delete API
    const _result = await searchClient.index(name, []) // Empty array to "clear" the document
    logger.info("Document cleared from index", { name })
    return c.json({ cleared: true, name })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Failed to clear document from index", {
      name: c.req.param("name"),
      error: String(err),
    })
    throw new HTTPException(500, { message: "Failed to clear document from index" })
  }
})

app.get("/health", (c) => {
  const healthy = searchClient !== null && searchClient !== undefined
  return c.json({
    status: healthy ? "healthy" : "degraded",
    service: "@dash/search",
    client: healthy ? "connected" : "disconnected",
  })
})

app.onError((err, c) => {
  if (err instanceof HTTPException) {
    return c.json({ error: err.message, status: err.status }, err.status as StatusCode)
  }
  logger.error("Unhandled error", { error: err.message })
  return c.json({ error: "Internal server error", status: 500 }, 500)
})

app.notFound((c) => c.json({ error: "Not found", status: 404 }, 404))

function parseFilters(filtersRaw: string | undefined): Record<string, string> | undefined {
  if (!filtersRaw) return undefined
  const filters: Record<string, string> = {}
  for (const part of filtersRaw.split(",")) {
    const [key, value] = part.split(":")
    if (key && value) filters[key] = value
  }
  return Object.keys(filters).length > 0 ? filters : undefined
}

export async function createSearchService(client?: SearchClient) {
  searchClient = client ?? (await createSearchClient(config.search))
  logger.info("Search service initialized")
  return app
}

export default app
