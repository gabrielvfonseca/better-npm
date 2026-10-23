export type FlagName = string

export interface FlagRule {
  field: string
  operator: "eq" | "neq" | "in" | "nin" | "gt" | "gte" | "lt" | "lte" | "contains"
  value: unknown
}

export interface FeatureFlag {
  name: string
  enabled: boolean
  rules?: FlagRule[]
}

function evaluateRule(rule: FlagRule, context: Record<string, unknown>): boolean {
  const contextValue = context[rule.field]

  switch (rule.operator) {
    case "eq":
      return contextValue === rule.value
    case "neq":
      return contextValue !== rule.value
    case "in":
      return Array.isArray(rule.value) && (rule.value as unknown[]).includes(contextValue)
    case "nin":
      return Array.isArray(rule.value) && !(rule.value as unknown[]).includes(contextValue)
    case "gt":
      return (
        typeof contextValue === "number" &&
        typeof rule.value === "number" &&
        contextValue > rule.value
      )
    case "gte":
      return (
        typeof contextValue === "number" &&
        typeof rule.value === "number" &&
        contextValue >= rule.value
      )
    case "lt":
      return (
        typeof contextValue === "number" &&
        typeof rule.value === "number" &&
        contextValue < rule.value
      )
    case "lte":
      return (
        typeof contextValue === "number" &&
        typeof rule.value === "number" &&
        contextValue <= rule.value
      )
    case "contains":
      return (
        typeof contextValue === "string" &&
        typeof rule.value === "string" &&
        contextValue.includes(rule.value)
      )
    default:
      return false
  }
}

export class FeatureFlags {
  private flags = new Map<FlagName, FeatureFlag>()

  constructor(initialFlags?: FeatureFlag[]) {
    if (initialFlags) {
      for (const flag of initialFlags) {
        this.flags.set(flag.name, flag)
      }
    }
  }

  isEnabled(name: FlagName, context: Record<string, unknown> = {}): boolean {
    const flag = this.flags.get(name)
    if (!flag) return false

    if (!flag.enabled) return false

    if (flag.rules && flag.rules.length > 0) {
      return flag.rules.every((rule) => evaluateRule(rule, context))
    }

    return true
  }

  getAll(): Record<FlagName, boolean> {
    const result: Record<FlagName, boolean> = {}
    for (const [name, flag] of this.flags) {
      result[name] = flag.enabled
    }
    return result
  }

  setFlag(name: FlagName, enabled: boolean): void {
    const existing = this.flags.get(name)
    if (existing) {
      existing.enabled = enabled
    } else {
      this.flags.set(name, { name, enabled })
    }
  }

  addFlag(flag: FeatureFlag): void {
    this.flags.set(flag.name, flag)
  }
}

export function createFeatureFlags(initialFlags?: FeatureFlag[]): FeatureFlags {
  return new FeatureFlags(initialFlags)
}
