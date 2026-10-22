import type {
  AgentClient,
  AgentClientConfig,
  ApplyResponse,
  AuditRequest,
  AuditResponse,
  ComposeRequest,
  ComposeResponse,
  ExplainResponse,
  PlanUpgradeRequest,
  PlanUpgradeResponse,
  SelectRequest,
  SelectResponse,
} from "./types.js"

export class BnpmAgentClient implements AgentClient {
  private config: Required<AgentClientConfig>

  constructor(config: AgentClientConfig) {
    this.config = {
      registry: config.registry,
      identity: config.identity ?? "anonymous",
    }
  }

  private async fetchJson(path: string, init?: RequestInit): Promise<unknown> {
    const url = `${this.config.registry}${path}`
    const response = await fetch(url, {
      ...init,
      headers: {
        "Content-Type": "application/json",
        "User-Agent": `@better-npm/agent/${this.config.identity}`,
        ...init?.headers,
      },
    })

    if (!response.ok) {
      const body = await response.text().catch(() => "")
      throw new Error(`Agent request failed: ${response.status} ${response.statusText} — ${body}`)
    }

    return response.json() as Promise<unknown>
  }

  async select(request: SelectRequest): Promise<SelectResponse> {
    return this.fetchJson("/.well-known/bnpm/select", {
      method: "POST",
      body: JSON.stringify(request),
    }) as Promise<SelectResponse>
  }

  async explain(packageName: string): Promise<ExplainResponse> {
    return this.fetchJson(
      `/.well-known/bnpm/explain?package=${encodeURIComponent(packageName)}`,
    ) as Promise<ExplainResponse>
  }

  async planUpgrade(request: PlanUpgradeRequest): Promise<PlanUpgradeResponse> {
    return this.fetchJson("/.well-known/bnpm/plan-upgrade", {
      method: "POST",
      body: JSON.stringify(request),
    }) as Promise<PlanUpgradeResponse>
  }

  async audit(request?: AuditRequest): Promise<AuditResponse> {
    const params = request
      ? `?${new URLSearchParams(
          Object.entries(request)
            .filter(([, v]) => v !== undefined)
            .map(([k, v]) => [k, String(v)]),
        ).toString()}`
      : ""
    return this.fetchJson(`/.well-known/bnpm/audit${params}`) as Promise<AuditResponse>
  }

  async compose(request: ComposeRequest): Promise<ComposeResponse> {
    return this.fetchJson("/.well-known/bnpm/compose", {
      method: "POST",
      body: JSON.stringify(request),
    }) as Promise<ComposeResponse>
  }

  async apply(plan: PlanUpgradeResponse): Promise<ApplyResponse> {
    return this.fetchJson("/.well-known/bnpm/apply", {
      method: "POST",
      body: JSON.stringify(plan),
    }) as Promise<ApplyResponse>
  }
}
