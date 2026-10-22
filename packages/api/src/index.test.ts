import { describe, expect, it } from "vitest"
import {
  badRequest,
  created,
  forbidden,
  noContent,
  notFound,
  ok,
  serverError,
  unauthorized,
} from "./index.js"

describe("@better-npm/api", () => {
  describe("response helpers", () => {
    it("should create ok response", () => {
      const res = ok({ data: "test" })
      expect(res.status).toBe(200)
      expect(res.body).toEqual({ data: "test" })
    })

    it("should create created response", () => {
      const res = created({ id: "1" })
      expect(res.status).toBe(201)
    })

    it("should create noContent response", () => {
      const res = noContent()
      expect(res.status).toBe(204)
    })

    it("should create error responses", () => {
      expect(badRequest("bad").status).toBe(400)
      expect(notFound().status).toBe(404)
      expect(unauthorized().status).toBe(401)
      expect(forbidden().status).toBe(403)
      expect(serverError().status).toBe(500)
    })

    it("should include error codes in error responses", () => {
      expect(badRequest("msg").body.code).toBe("BAD_REQUEST")
      expect(notFound().body.code).toBe("NOT_FOUND")
      expect(serverError().body.code).toBe("INTERNAL_ERROR")
    })
  })
})
