import { describe, expect, it } from "vitest"
import { createToken, hashPassword, parseAuthHeader, verifyPassword, verifyToken } from "./index.js"

describe("@better-npm/auth", () => {
  const secret = "test-secret-key"

  describe("createToken and verifyToken", () => {
    it("should create and verify a valid token", async () => {
      const token = await createToken(
        { sub: "user-1", email: "test@example.com", roles: ["user"] },
        secret,
        "1h",
      )
      expect(token.split(".")).toHaveLength(3)

      const result = await verifyToken(token, secret)
      expect(result.ok).toBe(true)
      if (result.ok) {
        expect(result.value.sub).toBe("user-1")
        expect(result.value.email).toBe("test@example.com")
      }
    })

    it("should reject a tampered token", async () => {
      const token = await createToken({ sub: "user-1" }, secret)
      const parts = token.split(".")
      const tampered = `${parts[0]}.${parts[1]}.invalidsig`
      const result = await verifyToken(tampered, secret)
      expect(result.ok).toBe(false)
    })
  })

  describe("hashPassword and verifyPassword", () => {
    it("should hash and verify a password", async () => {
      const password = "my-secure-password"
      const hashed = await hashPassword(password)
      expect(hashed).toContain(":")

      const valid = await verifyPassword(password, hashed)
      expect(valid).toBe(true)

      const invalid = await verifyPassword("wrong-password", hashed)
      expect(invalid).toBe(false)
    })
  })

  describe("parseAuthHeader", () => {
    it("should extract Bearer token", () => {
      const result = parseAuthHeader("Bearer my-token")
      expect(result.ok).toBe(true)
      if (result.ok) expect(result.value).toBe("my-token")
    })

    it("should reject missing header", () => {
      const result = parseAuthHeader("")
      expect(result.ok).toBe(false)
    })

    it("should reject invalid format", () => {
      const result = parseAuthHeader("Basic token")
      expect(result.ok).toBe(false)
    })
  })
})
