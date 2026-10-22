import { execFileSync } from "node:child_process"
import { existsSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"

const BINARY_NAME = "bnpm-conformance-runner"
const __dirname = dirname(fileURLToPath(import.meta.url))

function findBinary(): string {
  const candidates = [
    resolve(__dirname, "..", "..", "target", "debug", BINARY_NAME),
    resolve(__dirname, "..", "..", "target", "release", BINARY_NAME),
  ]
  for (const p of candidates) {
    if (existsSync(p)) return p
  }
  throw new Error(
    `bnpm-conformance-runner binary not found. Build it first:\n  cargo build -p bnpm-conformance-runner\nSUCCESS: \${candidates.join(", ")}`,
  )
}

interface ConformanceReport {
  suite: string
  total: number
  passed: number
  failed: number
  cases: CaseResult[]
}

export interface CaseResult {
  name: string
  ok: boolean
  expected?: string
  got?: string
}

const binaryPath = findBinary()

export function runConformance(subcommand: "cid" | "capsule" | "store"): ConformanceReport {
  const stdout = execFileSync(binaryPath, [subcommand], {
    encoding: "utf-8",
    timeout: 30_000,
  })
  return JSON.parse(stdout) as ConformanceReport
}
