import { describe, it, expect } from "vitest"
import { compileMdx, extractFrontmatter, extractToc } from "./index.js"

describe("@dash/mdx", () => {
  describe("extractFrontmatter", () => {
    it("should extract frontmatter", () => {
      const source = `---
title: My Post
description: A description
tags: [react, mdx]
---

# Hello`
      const meta = extractFrontmatter(source)
      expect(meta["title"]).toBe("My Post")
      expect(meta["tags"]).toEqual(["react", "mdx"])
    })

    it("should return empty object for no frontmatter", () => {
      expect(extractFrontmatter("# No frontmatter")).toEqual({})
    })
  })

  describe("extractToc", () => {
    it("should extract table of contents", () => {
      const source = "# Title\n\n## Section 1\n\n### Subsection\n\n## Section 2"
      const toc = extractToc(source)
      expect(toc).toHaveLength(4)
      expect(toc[0]).toMatchObject({ level: 1, text: "Title" })
      expect(toc[1]).toMatchObject({ level: 2, text: "Section 1" })
    })
  })

  describe("compileMdx", () => {
    it("should compile basic MDX", async () => {
      const result = await compileMdx("# Hello\n\nThis is **bold**")
      expect(result).toContain("Heading")
      expect(result).toContain("strong")
    })
  })
})
