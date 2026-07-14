# `@dash/markdown`

Lightweight markdown-to-HTML renderer.

## Overview

Converts markdown to basic HTML supporting headings, bold, italic, code, unordered/ordered lists, blockquotes, links, and paragraphs. Also provides `renderMarkdownSafe()` (HTML-escaped output), `extractHeadings()`, and `extractMetadata()` (front matter parsing).

## Usage

```ts
import { renderMarkdown, extractHeadings, extractMetadata } from "@dash/markdown";

const html = renderMarkdown("# Hello\nThis is **bold** text.");
const headings = extractHeadings(markdown);
const meta = extractMetadata(markdown);
```
