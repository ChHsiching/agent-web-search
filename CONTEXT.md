# Agent Web Search

A free, unlimited, stable web-search tool for coding agents (Claude Code, Codex, ZCode),
exposed as a stdio MCP server. Searches the web and returns results with page-body extracts.

## Language

**Source**:
An upstream a search query is sent to. A source answers one search query.
Currently: DuckDuckGo via the `ddgs` library (which itself aggregates multiple
search Sources — bing, brave, yandex, etc.). See ADR-0006.
_Avoid_: engine, backend, provider.

**Fan-out**:
Sending one query to multiple Sources concurrently, then merging their results.
The unit of reliability — one source failing does not fail the search.
_Avoid_: multi-query, broadcast (broadcast implies one-to-many side effects; this is gather).

**Extract**:
The raw page body of a search result, truncated to a word limit. Not a summary.
The agent reads and interprets it; we do not pre-digest content.
_Avoid_: summary, digest, snippet.

**Snippet**:
The short (~20–30 word) result description returned by the search Source itself,
as opposed to the Extract we fetch from the result's page.
_Avoid_: summary, abstract.
