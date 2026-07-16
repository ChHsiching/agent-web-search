## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

Wire the pieces together into one end-to-end search call: take the five `web_search_prime` parameters, build the SearXNG query (query builder), fan it out across instances (fanout), parse the winning JSON response into raw results (result parser), run the page-body extractor on the top 3 results to produce their Extract (extract, gated by `content_size`), derive `site_name` and `favicon` for each result (URL derivation), and assemble the final output list of `{ title, url, summary, site_name, favicon }`.

This is the orchestration layer — it calls the three pure/transformation modules and the fanout module, deciding what runs in what order and how the top-3 extraction is parallelized. It does not introduce new domain logic; it composes what the dependency tickets built.

Per the spec, Extract is applied to the top 3 results only (hardcoded, not exposed as a parameter); the remaining results return just the Snippet from the source. `content_size` controls the word limit passed to the extractor (medium ≈ 500, high ≈ 2500).

## Acceptance criteria

- [ ] `search(params, fetcher, instances) -> Vec<SearchResult>` composes query-build → fanout → parse → top-3-extract → assemble.
- [ ] The top 3 results each carry an Extract in their `summary` field (page body, word-limited by `content_size`); results beyond the top 3 carry the source Snippet as their `summary`.
- [ ] `content_size` maps to the extractor word limit (medium ≈ 500 words, high ≈ 2500 words).
- [ ] Each result has `title`, `url`, `summary`, `site_name` (derived), `favicon` (derived).
- [ ] If a top-3 result's page fetch or extraction fails, that result degrades gracefully (falls back to the Snippet, or an empty summary) without failing the whole search.
- [ ] If fanout returns no source (all instances exhausted), the search returns a clear error, not a panic.
- [ ] Tested via the fake `Fetch` seam (instances + page fetches canned) covering: normal multi-result search, top-3 extraction applied, extraction failure degradation, empty results, source-exhaustion error.

## Blocked by

- #6 — fanout: concurrent fan-out + batched retry + health score (produces the raw SearXNG JSON this orchestrates).
- #4 — search: query builder + result parser + URL derivation (the transformations this composes).
- #3 — extract: page-body extraction via readabilityrs (the extractor called for the top 3).
