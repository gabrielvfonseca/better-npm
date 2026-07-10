export const metadata = {
  name: "@dash/docs",
  description: "Dash documentation site",
  version: "0.1.0",
}

export interface DocFrontmatter {
  title: string
  description?: string
  order?: number
  section?: string
  draft?: boolean
}

export interface DocSection {
  id: string
  title: string
  order: number
  pages: DocPage[]
}

export interface DocPage {
  id: string
  title: string
  slug: string
  draft?: boolean
}

export const docSections: DocSection[] = [
  {
    id: "getting-started",
    title: "Getting Started",
    order: 1,
    pages: [
      { id: "installation", title: "Installation", slug: "/docs/installation" },
      { id: "quick-start", title: "Quick Start", slug: "/docs/quick-start" },
      { id: "configuration", title: "Configuration", slug: "/docs/configuration" },
    ],
  },
  {
    id: "guides",
    title: "Guides",
    order: 2,
    pages: [
      { id: "creating-packages", title: "Creating Packages", slug: "/docs/creating-packages" },
      { id: "publishing", title: "Publishing", slug: "/docs/publishing" },
      { id: "workspaces", title: "Workspaces", slug: "/docs/workspaces" },
      { id: "ci-cd", title: "CI/CD Integration", slug: "/docs/ci-cd" },
    ],
  },
  {
    id: "reference",
    title: "Reference",
    order: 3,
    pages: [
      { id: "cli", title: "CLI Reference", slug: "/docs/cli" },
      { id: "api", title: "API Reference", slug: "/docs/api" },
      { id: "configuration-file", title: "Configuration File", slug: "/docs/config" },
    ],
  },
]
