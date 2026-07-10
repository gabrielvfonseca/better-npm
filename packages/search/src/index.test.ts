import { describe, it, expect, vi, beforeEach, afterEach } from "vitest"
import { SearchClient, createSearchClient } from "./index.js"

describe("@dash/search", () => {
  beforeEach(() => {
    vi.spyOn(globalThis, "fetch").mockImplementation(async (url: RequestInfo | URL) => {
      const urlStr = url.toString()
      if (urlStr.includes("search")) {
        return new Response(
          JSON.stringify({ hits: [{ id: "1", name: "test-pkg", _score: 1 }], total: 1, page: 1, hitsPerPage: 20 }),
          { status: 200 },
        )
      }
      if (urlStr.includes("documents")) {
        return new Response(null, { status: 200 })
      }
      return new Response(null, { status: 404 })
    })
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it("should create a client", () => {
    const client = createSearchClient({ endpoint: "http://localhost:7700" })
    expect(client).toBeInstanceOf(SearchClient)
  })

  it("should search", async () => {
    const client = new SearchClient({ endpoint: "http://localhost:7700" })
    const result = await client.search({ q: "test" })
    expect(result.hits).toHaveLength(1)
    expect(result.total).toBe(1)
  })

  it("should suggest", async () => {
    const client = new SearchClient({ endpoint: "http://localhost:7700" })
    const suggestions = await client.suggest("test")
    expect(suggestions).toHaveLength(1)
    expect(suggestions[0]).toMatchObject({ text: "test-pkg" })
  })

  it("should index documents", async () => {
    const client = new SearchClient({ endpoint: "http://localhost:7700" })
    await expect(client.index("packages", [{ id: "1", name: "test" }])).resolves.toBeUndefined()
  })

  it("should reject on error responses", async () => {
    vi.restoreAllMocks()
    vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response(null, { status: 500, statusText: "Internal Error" }))
    const client = new SearchClient({ endpoint: "http://localhost:7700" })
    await expect(client.search({ q: "test" })).rejects.toThrow()
  })
})
