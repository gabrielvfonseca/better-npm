import { describe, expect, it } from "vitest"
import { z } from "zod"
import { assert, ValidationError, isValid, validate } from "./index.js"

describe("@better-npm/validation", () => {
  const stringSchema = z.string()

  describe("validate", () => {
    it("should return ok for valid data", () => {
      const result = validate(stringSchema, "hello")
      expect(result.ok).toBe(true)
      if (result.ok) {
        expect(result.value).toBe("hello")
      }
    })

    it("should return error for invalid data", () => {
      const result = validate(stringSchema, 42)
      expect(result.ok).toBe(false)
      if (!result.ok) {
        expect(result.error).toBeInstanceOf(ValidationError)
        expect(result.error.issues.length).toBeGreaterThan(0)
      }
    })
  })

  describe("assert", () => {
    it("should return data for valid input", () => {
      expect(assert(stringSchema, "test")).toBe("test")
    })

    it("should throw for invalid input", () => {
      expect(() => assert(stringSchema, 123)).toThrow(ValidationError)
    })
  })

  describe("isValid", () => {
    it("should return true for valid data", () => {
      expect(isValid(stringSchema, "hello")).toBe(true)
    })

    it("should return false for invalid data", () => {
      expect(isValid(stringSchema, 42)).toBe(false)
    })
  })
})
