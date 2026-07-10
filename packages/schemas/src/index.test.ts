import { describe, it, expect } from "vitest"
import {
  packageNameSchema,
  semVerSchema,
  publishPackageSchema,
  paginationParamsSchema,
  createUserSchema,
  workerResultSchema,
} from "./index.js"

describe("@dash/schemas", () => {
  describe("packageNameSchema", () => {
    it("should accept valid package names", () => {
      expect(packageNameSchema.parse("lodash")).toBe("lodash")
      expect(packageNameSchema.parse("@scope/pkg")).toBe("@scope/pkg")
      expect(packageNameSchema.parse("pkg-name")).toBe("pkg-name")
    })

    it("should reject invalid package names", () => {
      expect(() => packageNameSchema.parse("")).toThrow()
      expect(() => packageNameSchema.parse("UPPERCASE")).toThrow()
    })
  })

  describe("semVerSchema", () => {
    it("should accept valid semver", () => {
      expect(semVerSchema.parse("1.0.0")).toBe("1.0.0")
      expect(semVerSchema.parse("0.0.1")).toBe("0.0.1")
      expect(semVerSchema.parse("2.1.3-beta.1")).toBe("2.1.3-beta.1")
    })

    it("should reject invalid semver", () => {
      expect(() => semVerSchema.parse("1.0")).toThrow()
      expect(() => semVerSchema.parse("abc")).toThrow()
    })
  })

  describe("publishPackageSchema", () => {
    it("should accept valid publish input", () => {
      const input = {
        name: "test-pkg",
        version: "1.0.0",
        dist: {
          tarball: "https://example.com/pkg.tgz",
          shasum: "abc123",
        },
      }
      const result = publishPackageSchema.parse(input)
      expect(result.name).toBe("test-pkg")
      expect(result.tag).toBe("latest")
    })
  })

  describe("createUserSchema", () => {
    it("should accept valid user data", () => {
      const input = { username: "testuser", email: "test@example.com", password: "password123" }
      expect(createUserSchema.parse(input)).toEqual(input)
    })

    it("should reject weak passwords", () => {
      expect(() => createUserSchema.parse({ username: "test", email: "a@b.com", password: "short" })).toThrow()
    })
  })

  describe("paginationParamsSchema", () => {
    it("should apply defaults", () => {
      expect(paginationParamsSchema.parse({})).toEqual({ page: 1, pageSize: 20 })
    })
  })

  describe("workerResultSchema", () => {
    it("should accept success result", () => {
      expect(workerResultSchema.parse({ status: "success" })).toEqual({ status: "success" })
    })

    it("should accept failure result", () => {
      expect(workerResultSchema.parse({ status: "failure", error: "boom" })).toEqual({ status: "failure", error: "boom" })
    })
  })
})
