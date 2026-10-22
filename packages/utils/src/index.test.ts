import { describe, expect, it } from "vitest"
import {
  capitalize,
  clamp,
  cn,
  formatBytes,
  formatDate,
  isObject,
  pluralize,
  range,
  truncate,
} from "./index.js"

describe("@better-npm/utils", () => {
  describe("cn", () => {
    it("should join class names", () => {
      expect(cn("a", "b")).toBe("a b")
    })

    it("should filter falsy values", () => {
      expect(cn("a", false, null, undefined, 0, "b")).toBe("a b")
    })

    it("should handle object syntax", () => {
      expect(cn("base", { active: true, disabled: false })).toBe("base active")
    })

    it("should handle nested arrays", () => {
      expect(cn("a", ["b", ["c"]])).toBe("a b c")
    })
  })

  describe("formatBytes", () => {
    it("should format bytes", () => {
      expect(formatBytes(0)).toBe("0 Bytes")
      expect(formatBytes(1024)).toBe("1 KB")
      expect(formatBytes(1048576)).toBe("1 MB")
    })
  })

  describe("formatDate", () => {
    it("should format a date", () => {
      const date = new Date("2024-01-15")
      expect(formatDate(date)).toContain("Jan")
    })
  })

  describe("truncate", () => {
    it("should truncate strings", () => {
      expect(truncate("hello world", 5)).toBe("he...")
      expect(truncate("short", 10)).toBe("short")
    })
  })

  describe("capitalize", () => {
    it("should capitalize strings", () => {
      expect(capitalize("hello")).toBe("Hello")
      expect(capitalize("")).toBe("")
    })
  })

  describe("pluralize", () => {
    it("should pluralize words", () => {
      expect(pluralize(1, "package")).toBe("package")
      expect(pluralize(2, "package")).toBe("packages")
    })
  })

  describe("range", () => {
    it("should generate a range", () => {
      expect(range(0, 3)).toEqual([0, 1, 2])
    })
  })

  describe("clamp", () => {
    it("should clamp values", () => {
      expect(clamp(5, 0, 10)).toBe(5)
      expect(clamp(-5, 0, 10)).toBe(0)
      expect(clamp(15, 0, 10)).toBe(10)
    })
  })

  describe("isObject", () => {
    it("should detect objects", () => {
      expect(isObject({})).toBe(true)
      expect(isObject([])).toBe(false)
      expect(isObject(null)).toBe(false)
    })
  })
})
