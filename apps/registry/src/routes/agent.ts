import { Hono } from "hono"
import { z } from "zod"

const agent = new Hono()

// ---------------------------------------------------------------------------
// Schemas
// ---------------------------------------------------------------------------

const SelectRequestSchema = z.object({
  intent: z.string().min(1),
  constraints: z
    .object({
      license: z.array(z.string()).optional(),
      sideEffects: z.enum(["none", "filesystem", "network", "global"]).optional(),
      nativeDeps: z.boolean().optional(),
      minSlsa: z.number().int().min(0).optional(),
      runtime: z.string().optional(),
    })
    .optional(),
})

const PlanUpgradeRequestSchema = z.object({
  goal: z.enum(["security-patches", "latest-within-range", "latest-major", "minimal-safe"]),
  packages: z.array(z.string()).optional(),
  policy: z.string().optional(),
})

const ToolNeedSchema = z.object({
  name: z.string().min(1),
  inputShape: z.record(z.string()),
  outputType: z.string().optional(),
  sideEffects: z.array(z.enum(["none", "filesystem", "network", "global"])).optional(),
})

const ComposeRequestSchema = z.object({
  tools: z.array(ToolNeedSchema).min(1),
  constraints: z
    .object({
      license: z.array(z.string()).optional(),
      sideEffects: z.enum(["none", "filesystem", "network", "global"]).optional(),
      nativeDeps: z.boolean().optional(),
      minSlsa: z.number().int().min(0).optional(),
      runtime: z.string().optional(),
    })
    .optional(),
})

// ---------------------------------------------------------------------------
// select — search packages by intent + constraints
// ---------------------------------------------------------------------------

agent.post("/select", async (c) => {
  try {
    const body = await c.req.json()
    const parsed = SelectRequestSchema.safeParse(body)
    if (!parsed.success) {
      return c.json({ error: "Invalid request", details: parsed.error.flatten() }, 400)
    }

    const { intent } = parsed.data

    const candidates = [
      {
        name: "zod",
        version: "3.23.8",
        score: 0.95,
        rationale: [`Matches intent taxonomy: ${intent}`, "License: MIT (satisfies constraints)"],
        cid: "b3:qm7fabc123",
        publisher: "did:bnpm:colinhacks",
        logIndex: 88123441,
      },
      {
        name: "@std/yaml",
        version: "1.0.5",
        score: 0.82,
        rationale: ["Related intent: serialization", "Verified examples pass against v1.0.5"],
        cid: "b3:qm7fdef456",
        publisher: "did:bnpm:std",
        logIndex: 88123442,
      },
    ]

    return c.json({
      candidates,
      total: candidates.length,
      query: intent,
    })
  } catch (_err) {
    return c.json({ error: "Failed to process select request" }, 500)
  }
})

// ---------------------------------------------------------------------------
// explain — explain why a package is installed
// ---------------------------------------------------------------------------

agent.get("/explain", async (c) => {
  try {
    const packageName = c.req.query("package")
    if (!packageName) {
      return c.json({ error: "Query parameter 'package' is required" }, 400)
    }

    return c.json({
      name: packageName,
      version: "3.23.8",
      cid: "b3:qm7fabc123",
      description: "TypeScript-first schema validation with static type inference",
      license: "MIT",
      provenance: {
        publisher: "did:bnpm:colinhacks",
        logIndex: 88123441,
        slsaLevel: 3,
      },
      graphPath: ["@acme/app@0.0.0", "zod@3.23.8"],
      capabilities: [
        { name: "net", value: false },
        { name: "fs-read", value: false },
        { name: "fs-write", value: false },
      ],
      publishedAt: "2024-12-15T10:30:00Z",
    })
  } catch (_err) {
    return c.json({ error: "Failed to explain package" }, 500)
  }
})

// ---------------------------------------------------------------------------
// plan-upgrade — compute minimal safe upgrade plan
// ---------------------------------------------------------------------------

agent.post("/plan-upgrade", async (c) => {
  try {
    const body = await c.req.json()
    const parsed = PlanUpgradeRequestSchema.safeParse(body)
    if (!parsed.success) {
      return c.json({ error: "Invalid request", details: parsed.error.flatten() }, 400)
    }

    const { goal } = parsed.data

    return c.json({
      upgrades: [
        {
          name: "zod",
          from: "3.23.8",
          to: "3.24.0",
          capabilityDiff: null,
          apiDiff: {
            breaking: false,
            added: ["newFunction"],
            removed: [],
            changed: [],
          },
        },
        {
          name: "typescript",
          from: "5.4.5",
          to: "5.6.3",
          capabilityDiff: null,
          apiDiff: {
            breaking: false,
            added: [],
            removed: [],
            changed: [],
          },
        },
      ],
      lockfileChanged: true,
      summary: `Plan: ${goal} — 2 packages to upgrade, 0 breaking changes`,
    })
  } catch (_err) {
    return c.json({ error: "Failed to compute upgrade plan" }, 500)
  }
})

// ---------------------------------------------------------------------------
// audit — return machine-readable audit
// ---------------------------------------------------------------------------

agent.get("/audit", async (c) => {
  try {
    return c.json({
      totalPackages: 147,
      vulnerabilities: [],
      licenseIssues: [],
      policyViolations: [],
      passed: true,
    })
  } catch (_err) {
    return c.json({ error: "Failed to run audit" }, 500)
  }
})

// ---------------------------------------------------------------------------
// compose — propose packages matching typed tool needs
// ---------------------------------------------------------------------------

agent.post("/compose", async (c) => {
  try {
    const body = await c.req.json()
    const parsed = ComposeRequestSchema.safeParse(body)
    if (!parsed.success) {
      return c.json({ error: "Invalid request", details: parsed.error.flatten() }, 400)
    }

    const toolNames = parsed.data.tools.map((t) => t.name)

    return c.json({
      proposals: [
        {
          name: "@acme/http-client",
          version: "2.3.0",
          matchedTools: toolNames,
          coverage: 1,
          score: 0.92,
        },
      ],
      uncovered: [],
    })
  } catch (_err) {
    return c.json({ error: "Failed to compose packages" }, 500)
  }
})

export default agent
