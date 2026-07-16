## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

The pure data-transformation layer of search: convert the five `web_search_prime` parameters into a SearXNG query, parse SearXNG's JSON response into our result struct, and derive the per-result `site_name` and `favicon` fields from each result's URL. No network and no orchestration here — these are deterministic transformations, fully testable as pure functions.

Parameter mapping to SearXNG query params: `search_domain_filter` → `site:` query term; `search_recency_filter` (oneDay/oneWeek/oneMonth/oneYear/noLimit) → SearXNG `time_range`; `location` (cn/us) → SearXNG locale. `content_size` is consumed downstream by the extract step, not by the query builder.

URL derivation (ADR-0005): `site_name` = hostname with leading `www.` stripped; `favicon` = `{scheme}://{host}/favicon.ico` constructed from the result URL. Zero external dependency — a constructed string, never fetched by us.

## Acceptance criteria

- [ ] `build_searxng_query(params) -> (query_string, searxng_params)` correctly maps the five `web_search_prime` parameters to SearXNG's query and URL params (`q` with optional `site:`, `time_range`, `categories=general`, `format=json`, `locale`).
- [ ] `search_recency_filter` values map correctly: oneDay→`day`, oneWeek→`week`, oneMonth→`month`, oneYear→`year`, noLimit→absent.
- [ ] `parse_results(json: &str) -> Vec<RawResult>` parses SearXNG's JSON `results` array into `{ title, url, snippet, ... }`.
- [ ] `derive_site_name(url) -> String` strips leading `www.` and returns the host's main label.
- [ ] `derive_favicon(url) -> String` returns `{scheme}://{host}/favicon.ico`.
- [ ] Unit tests cover: each parameter mapping, the recency enum, JSON parsing of a realistic SearXNG response, URL derivation edge cases (with/without www, with path, with port).
- [ ] No network access occurs — all functions are pure.

## Blocked by

- None — can start immediately.
