# `@dash/mdx`

MDX compilation, front matter extraction, and table of contents generation.

## Overview

Converts MDX source to JSX-like output with heading components. Provides `extractFrontmatter()` for YAML front matter parsing and `extractToc()` for generating a table of contents with slugified heading IDs.

## Usage

```ts
import { compileMdx, extractFrontmatter, extractToc } from "@dash/mdx";

const { code } = await compileMdx(mdxSource, { components: { Heading } });
const frontmatter = extractFrontmatter(mdxSource);
const toc = extractToc(mdxSource);
```
