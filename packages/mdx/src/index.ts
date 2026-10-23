export interface MdxMeta {
  title?: string
  description?: string
  slug?: string
  tags?: string[]
  date?: string
  author?: string
}

export interface TocEntry {
  id: string
  text: string
  level: number
}

const HEADING_RE = /^(#{1,6})\s+(.+)$/gm
const FRONTMATTER_RE = /^---\n([\s\S]*?)\n---/

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .trim()
}

function parseYamlValue(value: string): string | string[] {
  const trimmed = value.trim()
  if (trimmed.startsWith("[")) {
    const inner = trimmed.slice(1, -1)
    return inner.split(",").map((s) => s.trim().replace(/^["']|["']$/g, ""))
  }
  return trimmed.replace(/^["']|["']$/g, "")
}

export async function compileMdx(source: string): Promise<string> {
  const cleaned = source.replace(FRONTMATTER_RE, "").trim()

  let jsx = cleaned
    .replace(
      /^### (.+)$/gm,
      'export const meta = { /* heading: "$1" */ }\n<Heading level={3}>$1</Heading>',
    )
    .replace(/^## (.+)$/gm, "<Heading level={2}>$1</Heading>")
    .replace(/^# (.+)$/gm, "<Heading level={1}>$1</Heading>")
    .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')

  jsx = jsx.replace(/^([^<\n].{0,80})$/gm, "<p>$1</p>")

  return `/* @jsx jsx */\n${jsx}`
}

export function extractFrontmatter(source: string): Record<string, unknown> {
  const match = FRONTMATTER_RE.exec(source)
  if (!match) return {}

  const frontmatter: Record<string, unknown> = {}
  const lines = match[1]?.split("\n") ?? []

  for (const line of lines) {
    const colonIdx = line.indexOf(":")
    if (colonIdx === -1) continue

    const key = line.slice(0, colonIdx).trim()
    const value = line.slice(colonIdx + 1).trim()

    if (!key) continue

    frontmatter[key] = parseYamlValue(value)
  }

  return frontmatter
}

export function extractToc(source: string): TocEntry[] {
  const entries: TocEntry[] = []
  const re = new RegExp(HEADING_RE)

  let match: RegExpExecArray | null = re.exec(source)
  while (match !== null) {
    const level = match[1]?.length
    const text = match[2]?.trim()
    entries.push({ level, text, id: slugify(text) })
    match = re.exec(source)
  }

  return entries
}
