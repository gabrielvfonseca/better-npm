import { describe, it, expect } from "vitest"
import {
  createContractRoute,
  getPackage,
  listPackages,
  publishPackage,
  searchPackages,
  login,
  register,
  getStats,
} from "./index.js"

describe("@dash/contracts", () => {
  it("should create a contract route", () => {
    const route = createContractRoute({
      path: "/test",
      method: "GET",
      input: {} as { id: string },
      output: {} as { name: string },
    })
    expect(route.path).toBe("/test")
    expect(route.method).toBe("GET")
  })

  it("should define getPackage contract", () => {
    expect(getPackage.path).toBe("/v1/packages/:name")
    expect(getPackage.method).toBe("GET")
  })

  it("should define listPackages contract", () => {
    expect(listPackages.path).toBe("/v1/packages")
    expect(listPackages.method).toBe("GET")
  })

  it("should define publishPackage contract", () => {
    expect(publishPackage.path).toBe("/v1/packages")
    expect(publishPackage.method).toBe("POST")
  })

  it("should define searchPackages contract", () => {
    expect(searchPackages.path).toBe("/v1/search")
    expect(searchPackages.method).toBe("GET")
  })

  it("should define auth contracts", () => {
    expect(login.path).toBe("/v1/auth/login")
    expect(login.method).toBe("POST")
    expect(register.path).toBe("/v1/auth/register")
    expect(register.method).toBe("POST")
  })

  it("should define stats contracts", () => {
    expect(getStats.path).toBe("/v1/stats")
    expect(getStats.method).toBe("GET")
  })
})
