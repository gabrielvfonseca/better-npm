import { describe, it, expect } from "vitest"
import { UserRole } from "./index.js"

describe("@dash/types", () => {
  it("should export UserRole enum with correct values", () => {
    expect(UserRole.Admin).toBe("admin")
    expect(UserRole.Moderator).toBe("moderator")
    expect(UserRole.Publisher).toBe("publisher")
    expect(UserRole.User).toBe("user")
  })

  it("should support branded types", () => {
    const id = "test" as import("./index.js").PackageId
    expect(typeof id).toBe("string")
  })
})
