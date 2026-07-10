import { describe, it, expect } from "vitest"
import { renderMarkdown, renderMarkdownSafe, extractHeadings, extractMetadata } from "./index.js"

describe("@dash/markdown", () => {
  describe("renderMarkdown", () => {
    it("should render headings", () => {
      expect(renderMarkdown("# Title")).toContain("<h1>")
      expect(renderMarkdown("## Subtitle")).toContain("<h2>")
      expect(renderMarkdown("### Subsection")).toContain("<h3>")
    })

    it("should render bold and italic", () => {
      expect(renderMarkdown("**bold**")).toContain("<strong>")
      expect(renderMarkdown("*italic*")).toContain("<em>")
    })

    it("should render links", () => {
      expect(renderMarkdown("[text](url)")).toContain('<a href="url">')
    })
  })

  describe("renderMarkdownSafe", () => {
    it("should escape HTML", () => {
      const result = renderMarkdownSafe("<script>alert('xss')</script>")
      expect(result).not.toContain("<script>")
      expect(result).toContain("&lt;script&gt;")
    })
  })

  describe("extractHeadings", () => {
    it("should extract headings from markdown", () => {
      const content = "# Title\n\nSome text\n\n## Section 1\n\n### Subsection\n\n## Section 2"
      const headings = extractHeadings(content)
      expect(headings).toHaveLength(4)
      expect(headings[0]).toMatchObject({ level: 1, text: "Title" })
      expect(headings[1]).toMatchObject({ level: 2, text: "Section 1" })
    })
  })

  describe("extractMetadata", () => {
    it("should extract frontmatter metadata", () => {
      const content = "---\ntitle: My Post\ndate: 2024-01-01\ntags: test\n---\n\nContent here"
      const metadata = extractMetadata(content)
      expect(metadata["title"]).toBe("My Post")
      expect(metadata["date"]).toBe("2024-01-01")
      expect(metadata["tags"]).toBe("test")
    })
  })
})
