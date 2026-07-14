// ============================================================================
// AI-Native Surface types — matching DESIGN.md Appendix E
// ============================================================================

/** Side-effect classification for packages */
export type SideEffect = "none" | "filesystem" | "network" | "global"

/** Stability level */
export type Stability = "experimental" | "stable" | "maintenance" | "deprecated"

// ---------------------------------------------------------------------------
// Select
// ---------------------------------------------------------------------------

export interface SelectConstraint {
  license?: string[]
  sideEffects?: SideEffect
  nativeDeps?: boolean
  minSlsa?: number
  runtime?: string
}

export interface SelectRequest {
  intent: string
  constraints?: SelectConstraint
}

export interface SelectResultPackage {
  name: string
  version: string
  score: number
  rationale: string[]
  cid?: string
  publisher?: string
  logIndex?: number
}

export interface SelectResponse {
  candidates: SelectResultPackage[]
  total: number
  query: string
}

// ---------------------------------------------------------------------------
// Explain
// ---------------------------------------------------------------------------

export interface ExplainRequest {
  packageName: string
}

export interface ExplainProvenance {
  publisher: string
  logIndex: number
  slsaLevel: number
}

export interface ExplainCapability {
  name: string
  value: string | string[] | boolean
}

export interface ExplainResponse {
  name: string
  version: string
  cid: string
  description?: string
  license?: string
  provenance?: ExplainProvenance
  graphPath: string[]
  capabilities: ExplainCapability[]
  publishedAt?: string
}

// ---------------------------------------------------------------------------
// PlanUpgrade
// ---------------------------------------------------------------------------

export type UpgradeGoal =
  | "security-patches"
  | "latest-within-range"
  | "latest-major"
  | "minimal-safe"

export interface PlanUpgradeRequest {
  goal: UpgradeGoal
  packages?: string[]
  policy?: string
}

export interface UpgradeItem {
  name: string
  from: string
  to: string
  capabilityDiff: Record<string, unknown> | null
  apiDiff: {
    breaking: boolean
    added: string[]
    removed: string[]
    changed: string[]
  } | null
}

export interface PlanUpgradeResponse {
  upgrades: UpgradeItem[]
  lockfileChanged: boolean
  summary: string
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

export interface AuditRequest {
  vulns?: boolean
  licenses?: boolean
  policy?: boolean
}

export interface Vulnerability {
  id: string
  severity: "critical" | "high" | "medium" | "low"
  package: string
  version: string
  description: string
}

export interface AuditResponse {
  totalPackages: number
  vulnerabilities: Vulnerability[]
  licenseIssues: string[]
  policyViolations: string[]
  passed: boolean
}

// ---------------------------------------------------------------------------
// Compose
// ---------------------------------------------------------------------------

export interface ToolNeed {
  name: string
  inputShape: Record<string, string>
  outputType?: string
  sideEffects?: SideEffect[]
}

export interface ComposeRequest {
  tools: ToolNeed[]
  constraints?: SelectConstraint
}

export interface ComposePackageProposal {
  name: string
  version: string
  matchedTools: string[]
  coverage: number
  score: number
}

export interface ComposeResponse {
  proposals: ComposePackageProposal[]
  uncovered: string[]
}

// ---------------------------------------------------------------------------
// Apply
// ---------------------------------------------------------------------------

export interface ApplyResponse {
  applied: boolean
  manifestChanged: boolean
  lockfileChanged: boolean
  verified: boolean
  diff: string
}

// ---------------------------------------------------------------------------
// AgentClient interface
// ---------------------------------------------------------------------------

export interface AgentClientConfig {
  registry: string
  identity?: string
}

export interface AgentClient {
  select(request: SelectRequest): Promise<SelectResponse>
  explain(packageName: string): Promise<ExplainResponse>
  planUpgrade(request: PlanUpgradeRequest): Promise<PlanUpgradeResponse>
  audit(request?: AuditRequest): Promise<AuditResponse>
  compose(request: ComposeRequest): Promise<ComposeResponse>
  apply(plan: PlanUpgradeResponse): Promise<ApplyResponse>
}
