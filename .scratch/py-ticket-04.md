## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

Wire the pieces into one end-to-end search call: take the five parameters, run the ddgs search (returns title/url/snippet), fetch page bodies for the top results, run the extractor on each (gated by `content_size`), derive `site_name` and `favicon`, and assemble the final output list of `{title, url, summary, site_name, favicon}`.

This is the orchestration layer — it composes the ddgs integration and the extract/derivation functions, deciding what runs in what order. Per the spec, Extract is applied to the top results only (hardcoded ~3, not exposed); the rest carry the source Snippet. `content_size` controls the extractor word limit (medium ≈ 500, high ≈ 2500). A page fetch/extraction failure degrades that single result to its Snippet without failing the whole search.

## Acceptance criteria

- [ ] `search(params)` composes ddgs-search → top-N-page-fetch → extract → assemble.
- [ ] Top results each carry a page-body Extract in `summary`; the rest carry the Snippet.
- [ ] `content_size` maps to the extractor word limit (medium≈500, high≈2500).
- [ ] Each result has `title`, `url`, `summary`, `site_name`, `favicon`.
- [ ] A page-fetch or extraction failure degrades that result to its Snippet; the search still succeeds.
- [ ] A ddgs failure (rate-limit, empty) returns a clear error, not a crash.
- [ ] Tested with ddgs + page-fetcher mocked, covering: normal multi-result search, top-N extraction applied, degradation on fetch failure, empty results.

## Blocked by

- #13 — ddgs integration: real search + parameter mapping (provides the search results this orchestrates).
- #12 — extract + content_size mapping + URL derivation (the functions this composes).
