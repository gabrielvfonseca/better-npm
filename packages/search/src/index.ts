export interface SearchQuery {
  q: string
  filters?: Record<string, unknown>
  sort?: "relevance" | "downloads" | "updated" | "published"
  page?: number
  pageSize?: number
}

export interface SearchResult {
  hits: Array<{
    id: string
    title: string
    description?: string
    score: number
    url: string
  }>
  total: number
  page: number
  pageSize: number
}

export interface SearchSuggestion {
  text: string
  score: number
}

export interface SearchConfig {
  endpoint?: string
  apiKey?: string
  indexPrefix?: string
}

export class SearchClient {
  private config: Required<SearchConfig>

  constructor(config: SearchConfig = {}) {
    this.config = {
      endpoint: config.endpoint ?? "http://localhost:7700",
      apiKey: config.apiKey ?? "",
      indexPrefix: config.indexPrefix ?? "",
    }
  }

  private getIndexName(name: string): string {
    return this.config.indexPrefix ? `${this.config.indexPrefix}_${name}` : name
  }

  async search(query: SearchQuery): Promise<SearchResult> {
    const params = new URLSearchParams({
      q: query.q,
      ...(query.page ? { page: String(query.page) } : {}),
      ...(query.pageSize ? { hitsPerPage: String(query.pageSize) } : {}),
    })

    const response = await fetch(
      `${this.config.endpoint}/indexes/${this.getIndexName("packages")}/search?${params}`,
      {
        headers: {
          ...(this.config.apiKey ? { Authorization: `Bearer ${this.config.apiKey}` } : {}),
          "Content-Type": "application/json",
        },
      },
    )

    if (!response.ok) {
      throw new Error(`Search failed: ${response.statusText}`)
    }

    const data = (await response.json()) as {
      hits: Array<Record<string, unknown>>
      total: number
      page: number
      hitsPerPage: number
    }

    return {
      hits: data.hits.map((hit) => ({
        id: String(hit["id"] ?? ""),
        title: String(hit["title"] ?? hit["name"] ?? ""),
        description: hit["description"] as string | undefined,
        score: Number(hit["_score"] ?? 1),
        url: String(hit["url"] ?? `/package/${hit["name"]}`),
      })),
      total: data.total,
      page: data.page,
      pageSize: data.hitsPerPage,
    }
  }

  async suggest(q: string): Promise<SearchSuggestion[]> {
    const params = new URLSearchParams({ q })

    const response = await fetch(
      `${this.config.endpoint}/indexes/${this.getIndexName("packages")}/search?${params}`,
      {
        headers: {
          ...(this.config.apiKey ? { Authorization: `Bearer ${this.config.apiKey}` } : {}),
          "Content-Type": "application/json",
        },
      },
    )

    if (!response.ok) {
      throw new Error(`Suggest failed: ${response.statusText}`)
    }

    const data = (await response.json()) as {
      hits: Array<Record<string, unknown>>
    }

    return data.hits.map((hit) => ({
      text: String(hit["name"] ?? hit["title"] ?? ""),
      score: Number(hit["_score"] ?? 1),
    }))
  }

  async index(name: string, documents: Record<string, unknown>[]): Promise<void> {
    const response = await fetch(
      `${this.config.endpoint}/indexes/${this.getIndexName(name)}/documents`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(this.config.apiKey ? { Authorization: `Bearer ${this.config.apiKey}` } : {}),
        },
        body: JSON.stringify(documents),
      },
    )

    if (!response.ok) {
      throw new Error(`Indexing failed: ${response.statusText}`)
    }
  }
}

export function createSearchClient(config?: SearchConfig): SearchClient {
  return new SearchClient(config)
}
