import { describe, it, expect, vi } from "vitest"
import { MemoryCache, createCache, memoize } from "./index.js"

describe("@dash/cache", () => {
  describe("MemoryCache", () => {
    it("should store and retrieve values", async () => {
      const cache = new MemoryCache()
      await cache.set("key", "value")
      expect(await cache.get("key")).toBe("value")
    })

    it("should return undefined for missing keys", async () => {
      const cache = new MemoryCache()
      expect(await cache.get("nonexistent")).toBeUndefined()
    })

    it("should respect TTL", async () => {
      vi.useFakeTimers()
      const cache = new MemoryCache()
      await cache.set("key", "value", 1)
      vi.advanceTimersByTime(1500)
      expect(await cache.get("key")).toBeUndefined()
      vi.useRealTimers()
    })

    it("should delete values", async () => {
      const cache = new MemoryCache()
      await cache.set("key", "value")
      await cache.delete("key")
      expect(await cache.has("key")).toBe(false)
    })

    it("should clear all values", async () => {
      const cache = new MemoryCache()
      await cache.set("a", 1)
      await cache.set("b", 2)
      await cache.clear()
      expect(await cache.get("a")).toBeUndefined()
      expect(await cache.get("b")).toBeUndefined()
    })

    it("should check existence", async () => {
      const cache = new MemoryCache()
      expect(await cache.has("key")).toBe(false)
      await cache.set("key", "value")
      expect(await cache.has("key")).toBe(true)
    })
  })

  describe("createCache", () => {
    it("should create a memory cache", () => {
      const cache = createCache({ backend: "memory", ttl: 60 })
      expect(cache).toBeInstanceOf(MemoryCache)
    })

    it("should throw for redis without client", () => {
      expect(() => createCache({ backend: "redis" })).toThrow()
    })
  })

  describe("memoize", () => {
    it("should cache function results", async () => {
      const fn = vi.fn().mockResolvedValue("result")
      const a = await memoize(fn, "key1")
      const b = await memoize(fn, "key1")
      expect(a).toBe("result")
      expect(b).toBe("result")
      expect(fn).toHaveBeenCalledTimes(1)
    })
  })
})
