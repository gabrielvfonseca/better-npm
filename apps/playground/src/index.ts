import { loadConfig } from "@dash/config"
import { createLogger } from "@dash/logger"
import { Hono } from "hono"
import { HTTPException } from "hono/http-exception"
import type { StatusCode } from "hono/utils/http-status"

const logger = createLogger("playground")
const _config = loadConfig()

const app = new Hono()

const rateLimitMap = new Map<string, { count: number; resetAt: number }>()
const RATE_LIMIT = 30
const RATE_WINDOW_MS = 60_000

function checkRateLimit(ip: string): void {
  const now = Date.now()
  const entry = rateLimitMap.get(ip)

  if (!entry || now > entry.resetAt) {
    rateLimitMap.set(ip, { count: 1, resetAt: now + RATE_WINDOW_MS })
    return
  }

  entry.count++
  if (entry.count > RATE_LIMIT) {
    throw new HTTPException(429, { message: "Rate limit exceeded. Try again later." })
  }
}

app.use("*", async (c, next) => {
  const start = performance.now()
  await next()
  const ms = (performance.now() - start).toFixed(2)
  logger.info(`${c.res.status} ${c.req.method} ${c.req.path} ${ms}ms`)
})

app.get("/playground", async (c) => {
  try {
    const html = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>@dash/playground</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0d1117; color: #c9d1d9; height: 100vh; display: flex; flex-direction: column; }
    header { padding: 16px 24px; border-bottom: 1px solid #30363d; display: flex; align-items: center; gap: 12px; }
    header h1 { font-size: 18px; font-weight: 600; }
    .container { display: flex; flex: 1; overflow: hidden; }
    .editor-panel { flex: 1; display: flex; flex-direction: column; border-right: 1px solid #30363d; }
    .output-panel { flex: 1; display: flex; flex-direction: column; }
    .panel-header { padding: 8px 16px; background: #161b22; border-bottom: 1px solid #30363d; font-size: 12px; text-transform: uppercase; letter-spacing: 0.05em; color: #8b949e; }
    textarea { flex: 1; background: #0d1117; color: #c9d1d9; border: none; padding: 16px; font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 14px; line-height: 1.6; resize: none; outline: none; }
    .output { flex: 1; padding: 16px; font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 14px; line-height: 1.6; overflow: auto; white-space: pre-wrap; }
    .toolbar { padding: 8px 16px; border-top: 1px solid #30363d; display: flex; gap: 8px; }
    button { background: #238636; color: #fff; border: none; padding: 8px 16px; border-radius: 6px; font-size: 14px; cursor: pointer; }
    button:hover { background: #2ea043; }
    button.secondary { background: #21262d; color: #c9d1d9; border: 1px solid #30363d; }
    button.secondary:hover { background: #30363d; }
    .error { color: #f85149; }
  </style>
</head>
<body>
  <header>
    <h1>@dash/playground</h1>
  </header>
  <div class="container">
    <div class="editor-panel">
      <div class="panel-header">Code</div>
      <textarea id="code" placeholder="Write JavaScript code here...">console.log("Hello, Dash!");</textarea>
      <div class="toolbar">
        <button id="runBtn">Run</button>
        <button id="formatBtn" class="secondary">Format</button>
      </div>
    </div>
    <div class="output-panel">
      <div class="panel-header">Output</div>
      <div id="output" class="output">Ready.</div>
    </div>
  </div>
  <script>
    const codeEl = document.getElementById('code');
    const outputEl = document.getElementById('output');
    const runBtn = document.getElementById('runBtn');
    const formatBtn = document.getElementById('formatBtn');

    runBtn.addEventListener('click', async () => {
      outputEl.textContent = 'Running...';
      outputEl.className = 'output';
      try {
        const res = await fetch('/playground/run', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ code: codeEl.value, language: 'javascript' })
        });
        const data = await res.json();
        if (data.error) {
          outputEl.textContent = data.error;
          outputEl.className = 'output error';
        } else {
          outputEl.textContent = data.output || '(no output)';
        }
      } catch (err) {
        outputEl.textContent = 'Error: ' + err.message;
        outputEl.className = 'output error';
      }
    });

    formatBtn.addEventListener('click', async () => {
      try {
        const res = await fetch('/playground/format', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ code: codeEl.value, language: 'javascript' })
        });
        const data = await res.json();
        if (data.error) {
          outputEl.textContent = data.error;
          outputEl.className = 'output error';
        } else {
          codeEl.value = data.formatted;
          outputEl.textContent = 'Code formatted successfully.';
        }
      } catch (err) {
        outputEl.textContent = 'Error: ' + err.message;
        outputEl.className = 'output error';
      }
    });
  </script>
</body>
</html>`

    return c.html(html)
  } catch (err) {
    logger.error("Failed to serve playground UI", { error: String(err) })
    throw new HTTPException(500, { message: "Failed to serve playground" })
  }
})

app.post("/playground/run", async (c) => {
  try {
    const ip = c.req.header("x-forwarded-for") ?? c.req.header("x-real-ip") ?? "unknown"
    checkRateLimit(ip)

    const body = await c.req.json()
    const { code, language } = body

    if (!code) {
      throw new HTTPException(400, { message: "Code is required" })
    }

    const result = await runInSandbox(code, language ?? "javascript")
    logger.info("Code executed", { language, ip })

    return c.json({ output: result.output, executionTime: result.executionTime })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Code execution failed", { error: String(err) })
    return c.json({ error: `Code execution failed: ${String(err)}` }, 500)
  }
})

app.post("/playground/format", async (c) => {
  try {
    const body = await c.req.json()
    const { code, language } = body

    if (!code) {
      throw new HTTPException(400, { message: "Code is required" })
    }

    const formatted = await formatCode(code, language ?? "javascript")
    return c.json({ formatted, language })
  } catch (err) {
    if (err instanceof HTTPException) throw err
    logger.error("Code formatting failed", { error: String(err) })
    return c.json({ error: `Code formatting failed: ${String(err)}` }, 500)
  }
})

app.get("/health", (c) => c.json({ status: "healthy", service: "@dash/playground" }))

app.onError((err, c) => {
  if (err instanceof HTTPException) {
    return c.json({ error: err.message, status: err.status }, err.status as StatusCode)
  }
  logger.error("Unhandled error", { error: err.message })
  return c.json({ error: "Internal server error", status: 500 }, 500)
})

app.notFound((c) => c.json({ error: "Not found", status: 404 }, 404))

async function runInSandbox(code: string, _language: string) {
  const start = performance.now()
  let output = ""

  try {
    const logs: string[] = []
    const mockConsole = {
      log: (...args: unknown[]) => logs.push(args.map(String).join(" ")),
      error: (...args: unknown[]) => logs.push(`Error: ${args.map(String).join(" ")}`),
      warn: (...args: unknown[]) => logs.push(`Warn: ${args.map(String).join(" ")}`),
      info: (...args: unknown[]) => logs.push(`Info: ${args.map(String).join(" ")}`),
    }

    const fn = new Function("console", code)
    fn(mockConsole)
    output = logs.join("\n")
  } catch (err) {
    output = `Execution error: ${String(err)}`
  }

  return { output, executionTime: performance.now() - start }
}

async function formatCode(code: string, language: string) {
  if (language === "javascript" || language === "typescript") {
    try {
      new Function(code)
      const lines = code.split("\n").filter((l) => l.trim())
      const indent = lines.map((l) => l.match(/^\s*/)?.[0]?.length ?? 0)
      const baseIndent = Math.min(...indent.filter((i) => i > 0))
      const formatted = lines
        .map((l) => l.slice(baseIndent))
        .join("\n")
        .trim()
      return formatted
    } catch {
      return code.trim()
    }
  }
  return code.trim()
}

export { app }
export default app
