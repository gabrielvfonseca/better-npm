export interface ToolProperty {
  type: string
  description?: string
  format?: string
  enum?: string[]
}

export interface ToolSchema {
  type: string
  properties?: Record<string, ToolProperty>
  required?: string[]
  description?: string
}

export interface ToolDefinition {
  name: string
  description: string
  input: ToolSchema
  output?: ToolSchema
  sideEffects?: string[]
}

export interface ToolInterfaceFile {
  tools: ToolDefinition[]
}

export interface ApiShapeFingerprint {
  packageName: string
  version: string
  intent?: string
  capabilitiesDeclared: string[]
  sideEffects?: string
  toolNames: string[]
  toolSignatures: string[]
  exportNames: string
}

export interface ApiShapeQuery {
  intent?: string
  toolName?: string
  hasTools?: boolean
  exportName?: string
  sideEffects?: string
  capability?: string
}

function cosineSimilarity(a: number[], b: number[]): number {
  const dot = a.reduce((sum, val, i) => sum + val * (b[i] ?? 0), 0)
  const magA = Math.sqrt(a.reduce((sum, val) => sum + val * val, 0))
  const magB = Math.sqrt(b.reduce((sum, val) => sum + val * val, 0))
  if (magA === 0 || magB === 0) return 0
  return dot / (magA * magB)
}

export class StructuralIndexer {
  private fingerprints = new Map<string, ApiShapeFingerprint>()

  indexPackage(
    packageName: string,
    version: string,
    params: {
      intent?: string
      capabilitiesDeclared?: string[]
      sideEffects?: string
      tools?: ToolDefinition[]
      exports?: string[]
    },
  ): void {
    const toolNames = (params.tools ?? []).map((t) => t.name).sort()
    const toolSignatures = (params.tools ?? [])
      .map((t) => {
        const params = Object.entries(t.input.properties ?? {})
          .map(([k, v]) => `${k}:${v.type}`)
          .sort()
          .join(",")
        return `${t.name}:(${params})`
      })
      .sort()

    this.fingerprints.set(packageName, {
      packageName,
      version,
      intent: params.intent,
      capabilitiesDeclared: params.capabilitiesDeclared ?? [],
      sideEffects: params.sideEffects,
      toolNames,
      toolSignatures,
      exportNames: (params.exports ?? []).sort().join(","),
    })
  }

  searchByApiShape(query: ApiShapeQuery): string[] {
    const scored: [string, number][] = []

    for (const [name, fp] of this.fingerprints) {
      let score = 0
      let criteria = 0

      if (query.intent) {
        criteria++
        if (fp.intent === query.intent) score++
        else if (fp.intent?.includes(query.intent)) score += 0.5
      }

      if (query.hasTools) {
        criteria++
        if (fp.toolNames.length > 0) score++
      }

      if (query.toolName) {
        criteria++
        if (fp.toolNames.includes(query.toolName)) score++
      }

      if (query.exportName) {
        criteria++
        if (fp.exportNames.includes(query.exportName)) score++
      }

      if (query.sideEffects) {
        criteria++
        if (fp.sideEffects === query.sideEffects) score++
      }

      if (query.capability) {
        criteria++
        if (fp.capabilitiesDeclared.includes(query.capability)) score++
      }

      if (criteria > 0 && score > 0) {
        scored.push([name, score / criteria])
      }
    }

    scored.sort((a, b) => b[1] - a[1])
    return scored.map(([name]) => name)
  }

  getFingerprint(packageName: string): ApiShapeFingerprint | undefined {
    return this.fingerprints.get(packageName)
  }

  clear(): void {
    this.fingerprints.clear()
  }

  get size(): number {
    return this.fingerprints.size
  }
}

export class SemanticIndex {
  private embeddings = new Map<string, number[]>()
  private texts = new Map<string, string>()

  private stubEmbed(text: string): number[] {
    const dims = 128
    const vec = new Array(dims).fill(0)
    for (let i = 0; i < text.length; i++) {
      const idx = i % dims
      vec[idx] = (vec[idx] ?? 0) + text.charCodeAt(i) / 255
    }
    const mag = Math.sqrt(vec.reduce((s, v) => s + v * v, 0))
    if (mag > 0) {
      for (let i = 0; i < vec.length; i++) {
        vec[i] = (vec[i] ?? 0) / mag
      }
    }
    return vec
  }

  index(packageId: string, text: string): void {
    this.embeddings.set(packageId, this.stubEmbed(text))
    this.texts.set(packageId, text)
  }

  search(query: string, k: number): [string, number][] {
    const queryVec = this.stubEmbed(query)
    const scores: [string, number][] = []

    for (const [id, emb] of this.embeddings) {
      scores.push([id, cosineSimilarity(queryVec, emb)])
    }

    scores.sort((a, b) => b[1] - a[1])
    return scores.slice(0, k)
  }

  remove(packageId: string): void {
    this.embeddings.delete(packageId)
    this.texts.delete(packageId)
  }

  get size(): number {
    return this.embeddings.size
  }
}
