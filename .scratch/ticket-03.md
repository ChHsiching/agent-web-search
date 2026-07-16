## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

A pure function that turns raw page HTML into a word-limited text Extract, using the `readabilityrs` crate (Mozilla modern Readability port — see ADR-0003). Given an HTML string and a word limit, it returns the main-content text truncated to that many words. No network, no I/O — a pure transformation, so it is fully testable without any HTTP seam.

This is the Extract the `summary` field of each result is built from (for the top 3 results), controlled by `content_size` (medium ≈ 500 words, high ≈ 2500 words). It is deliberately NOT a summary — it is raw page body, truncated by word count. The agent reads and interprets it; we do not pre-digest. See ADR-0001.

## Acceptance criteria

- [ ] `extract(html: &str, word_limit: usize) -> String` exists and returns main-content text extracted via `readabilityrs`, truncated to `word_limit` words.
- [ ] On a typical article HTML page, it strips navigation/sidebars/scripts and returns the article body text.
- [ ] Word truncation is by a reasonable word boundary (not mid-word), and respects the limit.
- [ ] Returns an empty string (not a panic) when given empty or unparseable HTML, or HTML with no detectable main content.
- [ ] Unit tests cover: normal article extraction, truncation at the limit, empty input, malformed HTML, HTML with no main content.
- [ ] No network access occurs — the function is pure.

## Blocked by

- None — can start immediately.
