import { loadConfig } from "@dash/config"
import { createLogger } from "@dash/logger"

const logger = createLogger("benchmarks")
const config = loadConfig()

interface BenchmarkResult {
  name: string
  category: string
  duration: number
  iterations: number
  opsPerSecond: number
  metrics?: Record<string, number>
}

interface Benchmark {
  name: string
  category: string
  run: () => Promise<BenchmarkResult> | BenchmarkResult
}

const benchmarks: Benchmark[] = [
  {
    name: "crypto-hash",
    category: "crypto",
    run: async () => {
      const iterations = 1000
      const start = performance.now()
      for (let i = 0; i < iterations; i++) {
        await crypto.subtle.digest("SHA-256", new TextEncoder().encode(`benchmark-${i}`))
      }
      const duration = performance.now() - start
      return {
        name: "crypto-hash",
        category: "crypto",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
  {
    name: "json-parse",
    category: "serialization",
    run: () => {
      const obj = { id: 1, name: "test", data: [1, 2, 3, 4, 5], nested: { a: 1, b: 2 } }
      const iterations = 10000
      const start = performance.now()
      for (let i = 0; i < iterations; i++) {
        JSON.parse(JSON.stringify(obj))
      }
      const duration = performance.now() - start
      return {
        name: "json-parse",
        category: "serialization",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
  {
    name: "array-sort",
    category: "algorithm",
    run: () => {
      const iterations = 1000
      const start = performance.now()
      for (let i = 0; i < iterations; i++) {
        const arr = Array.from({ length: 1000 }, () => Math.random())
        arr.sort((a, b) => a - b)
      }
      const duration = performance.now() - start
      return {
        name: "array-sort",
        category: "algorithm",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
  {
    name: "string-concat",
    category: "string",
    run: () => {
      const iterations = 50000
      const start = performance.now()
      let _result = ""
      for (let i = 0; i < iterations; i++) {
        _result += `item-${i},`
      }
      const duration = performance.now() - start
      return {
        name: "string-concat",
        category: "string",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
  {
    name: "regex-match",
    category: "string",
    run: () => {
      const text = "The quick brown fox jumps over the lazy dog 123 times."
      const iterations = 50000
      const start = performance.now()
      for (let i = 0; i < iterations; i++) {
        text.match(/[a-z]+/g)
      }
      const duration = performance.now() - start
      return {
        name: "regex-match",
        category: "string",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
  {
    name: "set-lookup",
    category: "data-structure",
    run: () => {
      const set = new Set(Array.from({ length: 10000 }, (_, i) => i))
      const iterations = 50000
      const start = performance.now()
      for (let i = 0; i < iterations; i++) {
        set.has(i % 10000)
      }
      const duration = performance.now() - start
      return {
        name: "set-lookup",
        category: "data-structure",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
  {
    name: "map-operations",
    category: "data-structure",
    run: () => {
      const map = new Map<string, number>()
      const iterations = 10000
      const start = performance.now()
      for (let i = 0; i < iterations; i++) {
        map.set(`key-${i}`, i)
        map.get(`key-${i}`)
        map.delete(`key-${i}`)
      }
      const duration = performance.now() - start
      return {
        name: "map-operations",
        category: "data-structure",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
  {
    name: "promise-all",
    category: "async",
    run: async () => {
      const iterations = 100
      const start = performance.now()
      for (let i = 0; i < iterations; i++) {
        await Promise.all([
          Promise.resolve(1),
          Promise.resolve(2),
          Promise.resolve(3),
          Promise.resolve(4),
          Promise.resolve(5),
        ])
      }
      const duration = performance.now() - start
      return {
        name: "promise-all",
        category: "async",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
  {
    name: "array-reduce",
    category: "algorithm",
    run: () => {
      const arr = Array.from({ length: 10000 }, (_, i) => i)
      const iterations = 1000
      const start = performance.now()
      for (let i = 0; i < iterations; i++) {
        arr.reduce((a, b) => a + b, 0)
      }
      const duration = performance.now() - start
      return {
        name: "array-reduce",
        category: "algorithm",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
  {
    name: "crypto-uuid",
    category: "crypto",
    run: () => {
      const iterations = 10000
      const start = performance.now()
      for (let i = 0; i < iterations; i++) {
        crypto.randomUUID()
      }
      const duration = performance.now() - start
      return {
        name: "crypto-uuid",
        category: "crypto",
        duration,
        iterations,
        opsPerSecond: (iterations / duration) * 1000,
      }
    },
  },
]

export function listBenchmarks(): string[] {
  return benchmarks.map((b) => b.name)
}

export async function runBenchmark(name: string): Promise<BenchmarkResult> {
  const benchmark = benchmarks.find((b) => b.name === name)
  if (!benchmark) {
    throw new Error(`Benchmark '${name}' not found. Available: ${listBenchmarks().join(", ")}`)
  }

  logger.info("Running benchmark", { name, category: benchmark.category })
  const result = await benchmark.run()
  logger.info("Benchmark completed", {
    name,
    duration: result.duration,
    opsPerSecond: result.opsPerSecond,
  })

  return result
}

export async function runBenchmarks(): Promise<{
  results: BenchmarkResult[]
  summary: { total: number; totalDuration: number; avgOpsPerSecond: number }
}> {
  logger.info("Running all benchmarks", { count: benchmarks.length })
  const start = performance.now()

  const results: BenchmarkResult[] = []
  for (const benchmark of benchmarks) {
    try {
      const result = await benchmark.run()
      results.push(result)
    } catch (error) {
      logger.error("Benchmark failed", { name: benchmark.name, error: String(error) })
      results.push({
        name: benchmark.name,
        category: benchmark.category,
        duration: 0,
        iterations: 0,
        opsPerSecond: 0,
        metrics: { error: 1 },
      })
    }
  }

  const totalDuration = performance.now() - start
  const validResults = results.filter((r) => r.opsPerSecond > 0)
  const avgOpsPerSecond =
    validResults.length > 0
      ? validResults.reduce((sum, r) => sum + r.opsPerSecond, 0) / validResults.length
      : 0

  const summary = { total: results.length, totalDuration, avgOpsPerSecond }

  logger.info("All benchmarks completed", summary)

  if (config.benchmarks?.outputResults) {
    const outputPath = config.benchmarks?.outputPath ?? "benchmarks-results.json"
    const fs = await import("node:fs/promises")
    await fs.writeFile(
      outputPath,
      JSON.stringify({ results, summary, timestamp: new Date().toISOString() }, null, 2),
    )
    logger.info("Benchmark results written", { path: outputPath })
  }

  return { results, summary }
}
