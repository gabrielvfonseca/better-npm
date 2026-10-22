import { describe, expect, it, vi } from "vitest"
import {
  EventEmitter,
  chunk,
  debounce,
  deepMerge,
  omit,
  pick,
  randomId,
  retry,
  sleep,
  throttle,
  timeout,
} from "./index.js"

describe("@better-npm/shared", () => {
  describe("randomId", () => {
    it("should generate a string of the specified length", () => {
      expect(randomId(8)).toHaveLength(8)
      expect(randomId(16)).toHaveLength(16)
    })

    it("should generate unique values", () => {
      const a = randomId()
      const b = randomId()
      expect(a).not.toBe(b)
    })
  })

  describe("omit", () => {
    it("should omit specified keys", () => {
      const obj = { a: 1, b: 2, c: 3 }
      expect(omit(obj, "a", "b")).toEqual({ c: 3 })
    })
  })

  describe("pick", () => {
    it("should pick specified keys", () => {
      const obj = { a: 1, b: 2, c: 3 }
      expect(pick(obj, "a", "c")).toEqual({ a: 1, c: 3 })
    })
  })

  describe("deepMerge", () => {
    it("should deeply merge objects", () => {
      const a: { x: { y: number; z?: number } } = { x: { y: 1, z: 2 } }
      const b = { x: { y: 3 } }
      expect(deepMerge(a, b)).toEqual({ x: { y: 3, z: 2 } })
    })
  })

  describe("retry", () => {
    it("should retry on failure and eventually succeed", async () => {
      let attempts = 0
      const fn = async () => {
        attempts++
        if (attempts < 3) throw new Error("fail")
        return "success"
      }
      await expect(retry(fn, { retries: 3, delay: 10 })).resolves.toBe("success")
      expect(attempts).toBe(3)
    })
  })

  describe("timeout", () => {
    it("should reject if promise takes too long", async () => {
      await expect(timeout(sleep(100), 10)).rejects.toThrow("Timed out")
    })

    it("should resolve if promise completes in time", async () => {
      await expect(timeout(Promise.resolve("ok"), 100)).resolves.toBe("ok")
    })
  })

  describe("chunk", () => {
    it("should chunk an array", () => {
      expect(chunk([1, 2, 3, 4, 5], 2)).toEqual([[1, 2], [3, 4], [5]])
    })
  })

  describe("debounce", () => {
    it("should delay invocation", async () => {
      vi.useFakeTimers()
      const fn = vi.fn()
      const debounced = debounce(fn, 100)
      debounced()
      debounced()
      debounced()
      expect(fn).not.toHaveBeenCalled()
      vi.advanceTimersByTime(100)
      expect(fn).toHaveBeenCalledTimes(1)
      vi.useRealTimers()
    })
  })

  describe("throttle", () => {
    it("should limit invocation rate", async () => {
      vi.useFakeTimers()
      const fn = vi.fn()
      const throttled = throttle(fn, 100)
      throttled()
      throttled()
      expect(fn).toHaveBeenCalledTimes(1)
      vi.advanceTimersByTime(100)
      expect(fn).toHaveBeenCalledTimes(2)
      vi.useRealTimers()
    })
  })

  describe("EventEmitter", () => {
    it("should emit and receive events", () => {
      const ee = new EventEmitter()
      const handler = vi.fn()
      ee.on("test", handler)
      ee.emit("test", 1, 2)
      expect(handler).toHaveBeenCalledWith(1, 2)
    })

    it("should support once", () => {
      const ee = new EventEmitter()
      const handler = vi.fn()
      ee.once("test", handler)
      ee.emit("test")
      ee.emit("test")
      expect(handler).toHaveBeenCalledTimes(1)
    })

    it("should support off", () => {
      const ee = new EventEmitter()
      const handler = vi.fn()
      ee.on("test", handler)
      ee.off("test", handler)
      ee.emit("test")
      expect(handler).not.toHaveBeenCalled()
    })
  })
})
