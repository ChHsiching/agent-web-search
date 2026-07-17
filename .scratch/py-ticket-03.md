## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

Pure functions for page-body extraction and per-result derivation — no network, no I/O, fully testable standalone:

1. **Extract:** given raw page HTML and a word limit, return the main-content plain text truncated to that many words, using a Python Readability port (`readability-lxml` or equivalent). This populates the `summary` field for top results. It is deliberately NOT a summary — raw page body, truncated. See ADR-0001.
2. **content_size → word limit:** map "medium"→~500 words, "high"→~2500 words (default medium).
3. **URL derivation (ADR-0005):** `derive_site_name(url)` (strip leading `www.`, take host) and `derive_favicon(url)` (`{scheme}://{host}/favicon.ico`), zero external dependency.

## Acceptance criteria

- [ ] `extract(html, word_limit)` returns main-content text truncated to `word_limit` words; strips nav/sidebar/script noise.
- [ ] Returns empty string (no crash) on empty, malformed, or no-main-content HTML.
- [ ] `word_limit_for(content_size)` maps medium→500, high→2500, None/unknown→500.
- [ ] `derive_site_name(url)` strips `www.` and returns the host's main label; handles missing/odd URLs.
- [ ] `derive_favicon(url)` returns `{scheme}://{host}/favicon.ico`; empty on non-URL input.
- [ ] Unit tests cover: extraction, truncation, empty/malformed input, content_size mapping, URL derivation edge cases.
- [ ] No network access — all functions are pure.

## Blocked by

- None — can start immediately.
