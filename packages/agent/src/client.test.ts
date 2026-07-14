import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { createAgentClient } from "./index.js"
import type {
  ApplyResponse,
  AuditResponse,
  ComposeResponse,
  ExplainResponse,
  PlanUpgradeResponse,
  SelectResponse,
} from "./types.js"

const mockSelectResponse: SelectResponse = {
  candidates: [
    { name: "zod", version: "3.23.8", score: 0.95, rationale: ["Matches intent validation"] },
  ],
  total: 1,
  query: "validation",
}

const mockExplainResponse: ExplainResponse = {
  name: "zod",
  version: "3.23.8",
  cid: "b3:qm7fabc123",
  description: "TypeScript-first schema validation",
  license: "MIT",
  provenance: { publisher: "did:dash:colinhacks", logIndex: 88123441, slsaLevel: 3 },
  graphPath: ["@acme/app@0.0.0", "zod@3.23.8"],
  capabilities: [{ name: "net", value: false }],
}

const mockPlanUpgradeResponse: PlanUpgradeResponse = {
  upgrades: [
    {
      name: "zod",
      from: "3.23.8",
      to: "3.24.0",
      capabilityDiff: null,
      apiDiff: { breaking: false, added: ["newFn"], removed: [], changed: [] },
    },
  ],
  lockfileChanged: true,
  summary: "1 package to upgrade",
}

const mockAuditResponse: AuditResponse = {
  totalPackages: 147,
  vulnerabilities: [],
  licenseIssues: [],
  policyViolations: [],
  passed: true,
}

const mockComposeResponse: ComposeResponse = {
  proposals: [
    {
      name: "@acme/http-client",
      version: "2.3.0",
      matchedTools: ["request"],
      coverage: 1,
      score: 0.9,
    },
  ],
  uncovered: [],
}

const mockApplyResponse: ApplyResponse = {
  applied: true,
  manifestChanged: true,
  lockfileChanged: true,
  verified: true,
  diff: '--- a/dash.toml\n+++ b/dash.toml\n@@ -1 +1 @@\n-zod = "^3.23"\n+zod = "^3.24"',
}

describe("DashAgentClient", () => {
  let client: ReturnType<typeof createAgentClient>
  let fetchSpy: ReturnType<typeof vi.fn>

  beforeEach(() => {
    fetchSpy = vi.fn()
    vi.stubGlobal("fetch", fetchSpy)
    client = createAgentClient({ registry: "https://reg.dash.dev" })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it("should select packages", async () => {
    fetchSpy.mockResolvedValue(
      new Response(JSON.stringify(mockSelectResponse), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    )

    const result = await client.select({ intent: "validation", constraints: { license: ["MIT"] } })
    expect(result.candidates).toHaveLength(1)
    expect(result.candidates[0]?.name).toBe("zod")
    expect(fetchSpy).toHaveBeenCalledWith(
      "https://reg.dash.dev/.well-known/dash/select",
      expect.objectContaining({ method: "POST", body: expect.any(String) }),
    )
  })

  it("should explain a package", async () => {
    fetchSpy.mockResolvedValue(
      new Response(JSON.stringify(mockExplainResponse), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    )

    const result = await client.explain("zod")
    expect(result.name).toBe("zod")
    expect(result.provenance?.publisher).toBe("did:dash:colinhacks")
    expect(fetchSpy).toHaveBeenCalledWith(
      "https://reg.dash.dev/.well-known/dash/explain?package=zod",
      expect.any(Object),
    )
  })

  it("should plan upgrades", async () => {
    fetchSpy.mockResolvedValue(
      new Response(JSON.stringify(mockPlanUpgradeResponse), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    )

    const result = await client.planUpgrade({ goal: "security-patches" })
    expect(result.upgrades).toHaveLength(1)
    expect(result.upgrades[0]?.name).toBe("zod")
  })

  it("should audit", async () => {
    fetchSpy.mockResolvedValue(
      new Response(JSON.stringify(mockAuditResponse), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    )

    const result = await client.audit()
    expect(result.passed).toBe(true)
    expect(result.totalPackages).toBe(147)
  })

  it("should compose", async () => {
    fetchSpy.mockResolvedValue(
      new Response(JSON.stringify(mockComposeResponse), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    )

    const result = await client.compose({
      tools: [{ name: "request", inputShape: { url: "string" } }],
    })
    expect(result.proposals).toHaveLength(1)
  })

  it("should apply a plan", async () => {
    fetchSpy.mockResolvedValue(
      new Response(JSON.stringify(mockApplyResponse), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    )

    const result = await client.apply(mockPlanUpgradeResponse)
    expect(result.applied).toBe(true)
    expect(result.verified).toBe(true)
  })

  it("should throw on error responses", async () => {
    fetchSpy.mockResolvedValue(new Response("Not found", { status: 404, statusText: "Not Found" }))
    await expect(client.select({ intent: "test" })).rejects.toThrow(
      "Agent request failed: 404 Not Found",
    )
  })

  it("should include User-Agent header", async () => {
    fetchSpy.mockResolvedValue(
      new Response(JSON.stringify(mockSelectResponse), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    )

    await client.select({ intent: "validation" })
    const callArgs = fetchSpy.mock.calls[0]
    const headers = (callArgs?.[1] as RequestInit)?.headers as Record<string, string> | undefined
    expect(headers?.["User-Agent"]).toContain("@dash/agent")
  })
})
