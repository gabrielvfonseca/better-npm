import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { BnpmClient, BnpmClientError, createClient } from "./index.js"

describe("@better-npm/client", () => {
  let client: BnpmClient

  beforeEach(() => {
    vi.spyOn(globalThis, "fetch").mockImplementation(async (url: RequestInfo | URL) => {
      const urlStr = url.toString()
      if (urlStr.includes("/error")) {
        return new Response(
          JSON.stringify({ code: "TEST_ERROR", message: "Something went wrong" }),
          {
            status: 400,
            headers: { "Content-Type": "application/json" },
          },
        )
      }
      if (urlStr.includes("/empty")) {
        return new Response(null, { status: 204 })
      }
      return new Response(JSON.stringify({ ok: true, data: "test" }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      })
    })

    client = createClient({ baseUrl: "https://api.example.com", token: "test-token" })
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it("should create a client", () => {
    expect(client).toBeInstanceOf(BnpmClient)
  })

  it("should make GET requests", async () => {
    const result = await client.get<{ ok: boolean; data: string }>("/test")
    expect(result.ok).toBe(true)
    expect(result.data).toBe("test")
  })

  it("should make POST requests", async () => {
    const result = await client.post<{ ok: boolean }>("/test", { key: "value" })
    expect(result.ok).toBe(true)
  })

  it("should make PUT requests", async () => {
    const result = await client.put<{ ok: boolean }>("/test", { key: "value" })
    expect(result.ok).toBe(true)
  })

  it("should make DELETE requests", async () => {
    const result = await client.delete<{ ok: boolean }>("/test")
    expect(result.ok).toBe(true)
  })

  it("should make PATCH requests", async () => {
    const result = await client.patch<{ ok: boolean }>("/test", { key: "value" })
    expect(result.ok).toBe(true)
  })

  it("should handle 204 no content", async () => {
    const result = await client.get<undefined>("/empty")
    expect(result).toBeUndefined()
  })

  it("should throw on error responses", async () => {
    await expect(client.get("/error")).rejects.toThrow(BnpmClientError)
  })

  it("should include auth token", async () => {
    const fetchSpy = vi.mocked(fetch)
    await client.get("/test")
    const headers = fetchSpy.mock.calls[0]?.[1]?.headers as Record<string, string>
    expect(headers.Authorization).toBe("Bearer test-token")
  })

  it("should manage tokens", () => {
    client.setToken("new-token")
    expect(client.getToken()).toBe("new-token")
    client.setToken(undefined)
    expect(client.getToken()).toBeUndefined()
  })
})
