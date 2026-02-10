---
name: web-search
description: >
  Search the web using Bose Search Engine (SearXNG meta-search with 247 engines).
  Use when you need to find current information, documentation, code examples,
  news, or any web content. Supports category filtering (general, science, it, news),
  language selection, and time range filtering.
---

# Bose Web Search

Search the web via the `bose-search` MCP server.

## Available Tools

### web_search
Search the web with SearXNG (247 search engines aggregated).

**Parameters:**
- `query` (required): The search query string
- `num_results` (optional): Number of results, default 10
- `category` (optional): general, science, it, news
- `language` (optional): Language code (en, zh-TW, ja)
- `time_range` (optional): day, week, month, year

**Example usage:**
- "Search for Rust async runtime benchmarks 2026"
- "Find the latest tokio documentation"
- "Search for SIMD optimization techniques in Rust"

### health_check
Verify the SearXNG backend is running and responsive.

## Prerequisites

1. SearXNG container must be running:
   ```
   cd deploy && podman-compose up -d
   ```
2. MCP server binary must be built:
   ```
   cargo build --release -p bose-mcp
   ```

## Notes
- Results include title, URL, snippet, source engine, and category
- Multiple engines (Google, Brave, DuckDuckGo, Startpage, etc.) are queried in parallel
- Response time is typically 1-3 seconds
