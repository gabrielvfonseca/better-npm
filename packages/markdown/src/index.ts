export interface Heading {
  level: number
  text: string
  id: string
}

const HEADING_RE = /^(#{1,6})\s+(.+)$/gm
const METADATA_LINE_RE = /^(\w[\w-]*)\s*:\s*(.+)$/

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\w\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-")
    .trim()
}

export function renderMarkdown(content: string): string {
  let html = content

  html = html.replace(/^### (.+)$/gm, "<h3>$1</h3>")
  html = html.replace(/^## (.+)$/gm, "<h2>$1</h2>")
  html = html.replace(/^# (.+)$/gm, "<h1>$1</h1>")

  html = html.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
  html = html.replace(/\*(.+?)\*/g, "<em>$1</em>")
  html = html.replace(/`([^`]+)`/g, "<code>$1</code>")

  html = html.replace(/^-\s+(.+)$/gm, "<li>$1</li>")
  html = html.replace(/(<li>.*<\/li>\n?)+/g, "<ul>$&</ul>")

  html = html.replace(/^> (.+)$/gm, "<blockquote><p>$1</p></blockquote>")

  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')

  html = html.replace(/^([^<\n].+)$/gm, "<p>$1</p>")

  return html.trim()
}

export function renderMarkdownSafe(content: string): string {
  const escaped = content
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
  return renderMarkdown(escaped)
}

export function extractHeadings(content: string): Heading[] {
  const headings: Heading[] = []
  const re = new RegExp(HEADING_RE)

  let match: RegExpExecArray | null = re.exec(content)
  while (match !== null) {
    const level = match[1]?.length
    const text = match[2]?.trim()
    headings.push({ level, text, id: slugify(text) })
    match = re.exec(content)
  }

  return headings
}

export function extractMetadata(content: string): Record<string, string> {
  const metadata: Record<string, string> = {}
  const lines = content.split("\n")
  let inFrontMatter = false

  for (const line of lines) {
    const trimmed = line.trim()

    if (trimmed === "---") {
      inFrontMatter = !inFrontMatter
      continue
    }

    if (inFrontMatter) {
      const match = METADATA_LINE_RE.exec(trimmed)
      if (match) {
        const key = match[1]
        if (key) {
          metadata[key] = match[2]?.trim() ?? ""
        }
      }
    } else if (!trimmed) {
      break
    }
  }

  return metadata
}
