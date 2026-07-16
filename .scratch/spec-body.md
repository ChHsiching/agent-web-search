## Problem Statement

Coding agents (Claude Code, Codex, ZCode) need web search to look up docs, APIs, libraries, and error solutions. The official/hosted web-search MCP tools they ship with are **paid** (per-query billing, monthly quotas). A developer who just wants their agent to search the web hits a paywall or a `429 Weekly/Monthly Limit Exhausted` — exactly the failure that motivated this project (the built-in `web_search_prime` tool returned that error mid-session).

The user wants a **free, unlimited, stable, drop-in replacement** for the paid `web_search_prime` MCP tool — same tool name, same parameters — so switching is a config change, not a code change.

## Solution

A **single static-binary stdio MCP server**, written in **Rust** (with `tokio`), that exposes one tool — `web_search_prime` — backed entirely by **free SearXNG public instances** queried via their JSON API.

- The server is installed by downloading one pre-compiled binary; no Docker, no Python, no `npx`, no API key, no runtime fee.
- Reliability comes from **fan-out across many instances** (concurrently query the top 3 healthiest, take the fastest success) plus **batched retry** down the eligible list (~43 healthy instances), never from a single source.
- The instance list is **fetched live from `searx.space`** (auto-filtered to healthy instances), cached locally, and never hand-maintained.
- Results include **page-body Extract** (raw text, truncated by `content_size`) for the top 3 results — the agent reads and interprets it; we do **no summarization**.
- The MCP connection **never fails to establish** at startup: the `initialize` handshake waits on no network, stdout carries only JSON-RPC, and startup never panics.

## User Stories

1. As a developer using Claude Code, I want to replace my paid `web_search_prime` MCP with a free one by changing one config line, so that I don't pay per search.
2. As a developer using Codex, I want the replacement to expose the same tool name and parameters, so that my agent's prompts and tool calls work unchanged.
3. As a developer using ZCode, I want the same drop-in replacement, so that all three agents I use share one free tool.
4. As a developer on a low-spec machine (Intel 9th-gen-class), I want the tool to run without strain, so that it doesn't compete with my browser or IDE.
5. As a developer, I want to install by downloading a single binary, so that I don't need Docker, Python, or a Rust toolchain to use it.
6. As a developer, I want the tool to start in milliseconds, so that my agent connects without a cold-start timeout.
7. As a developer, I want the MCP connection to never fail at startup, so that my agent reliably discovers the tool.
8. As a developer, I want searches to keep working when one SearXNG instance is down, so that I am not dependent on any single volunteer-run server.
9. As a developer, I want searches to keep working when several instances are down, so that reliability scales with the size of the pool.
10. As a developer, I want the tool to prefer the fastest healthy instances, so that searches return quickly.
11. As a developer, I want dead instances to be skipped automatically, so that I never wait on a dead server.
12. As a developer, I want no manual list of instances to maintain, so that the tool stays current without my effort.
13. As a developer, I want the instance pool to stay up to date on its own, so that new healthy instances are used and retired ones drop off.
14. As a developer, I want no API key or registration required, so that setup is frictionless.
15. As a developer, I want to limit results to a specific domain (`search_domain_filter`), so that I can scope a search to one site.
16. As a developer, I want to filter by recency (`search_recency_filter`), so that I can find recent releases or breaking changes.
17. As a developer, I want a region parameter (`location`), so that results match my locale (cn/us).
18. As a developer, I want richer page content when I ask for it (`content_size=high`), so that I get enough context to judge a result without a separate fetch.
19. As an agent, I want the `summary` field to contain real page text, so that I can read and reason about it directly (not a downgraded digest).
20. As an agent, I want each result to include a title, url, site name, and favicon, so that the result schema matches `web_search_prime`.
21. As a maintainer, I want the core search logic testable without touching the network, so that I can verify fan-out, retry, and merge deterministically.
22. As a maintainer, I want the content extractor testable as a pure function, so that I can verify HTML-to-text without HTTP.
23. As a maintainer, I want the MCP transport isolated from core logic, so that I can test behavior without spinning up the protocol layer.
24. As a maintainer, I want stdout reserved for JSON-RPC, so that stray logging never corrupts the MCP frame stream.
25. As a maintainer, I want all logs on stderr, so that I can debug without risking the connection.
26. As a maintainer, I want a clear error returned when no instance is available, so that a query failure is distinguishable from a connection failure.
27. As a developer, I want the tool to work on Windows, Linux, and macOS (x64 and arm64), so that I can use it on any machine.
28. As a developer, I want a clear, simple installation doc, so that I can set it up in under a minute.

## Implementation Decisions

- **Single Rust binary, single crate.** Internal modules by responsibility: `main` (entry), `mcp` (protocol shell — the sole stdout owner), `search` (SearXNG query construction + JSON result parsing), `sources` (searx.space fetch, instance filtering/ranking, local cache, in-memory health score), `fanout` (concurrent fan-out + batched retry via tokio), `extract` (page-body extraction via `readabilityrs` + word-limit truncation), `config` (local cache paths). No workspace split — the codebase is small.
- **Language: Rust + `tokio`.** Compile-time safety (`Send`/`Sync`, `Result`/`?`) serves the stability-first goal; async fan-out is idiomatic. See ADR-0002.
- **Tool interface: 1:1 with `web_search_prime`.** Name `web_search_prime`; parameters `search_query` (required), `search_domain_filter`, `search_recency_filter` (oneDay|oneWeek|oneMonth|oneYear|noLimit), `content_size` (medium|high), `location` (cn|us). All optional ones map to SearXNG query params (`site:`, `time_range`, locale).
- **Search backend: SearXNG public instances only.** Query the JSON API (`?format=json`). No Brave, no Playwright, no self-scraping. See ADR-0001.
- **Instance list: live from `searx.space/data/instances.json`.** Auto-filter to `timing.search.success_percentage == 100 && http.status_code == 200`; JSON-API openness detected by a one-shot `format=json` probe; ranked by search-latency median. Locally cached (~1h TTL) so a later startup degrades to cache if searx.space is down. Never hand-maintained.
- **Rotation: concurrent fan-out to top 3, take first success, cancel the rest.** Per-instance timeout ~3–5s. On all-3 failure, retry against the next batch of 3, down the eligible list. Fails only when every eligible instance is exhausted. See ADR-0001.
- **Health score: in-memory, non-persisted.** Consecutive failures temporarily demote an instance so dead ones aren't retried as primaries. Resets on restart.
- **Fetch timing vs. handshake (amends ADR-0001 via ADR-0004):** The MCP `initialize` handshake replies immediately and waits on no network. The searx.space fetch runs post-handshake in the background. A query arriving with an empty in-memory list does one short synchronous fetch for that query only; failure returns a clear "no instances available" error (a query failure, never a connection failure).
- **No summarization.** `content_size` controls how many words of page body to return, not a generated/extractive summary. The agent reads raw text. See ADR-0001.
- **Extract applied to top 3 results only** (default, hardcoded, not exposed). Remaining results return just the Snippet from the source. Keeps the per-query fetch cost bounded.
- **Output struct per result:** `{ title, url, summary (the Extract — page body text), site_name, favicon }`. `site_name` and `favicon` derived from the URL (strip `www.`, take domain; `{scheme}://{host}/favicon.ico`). Zero external dependency for favicon. See ADR-0005.
- **Content extraction library: `readabilityrs`** (Mozilla modern Readability port, actively maintained). See ADR-0003.
- **Distribution: GitHub Releases pre-compiled binaries** (Windows/Linux/macOS × x64/arm64) as the primary install path; `cargo install` via crates.io as a secondary path.
- **MCP stability disciplines (non-negotiable, ADR-0004):** stdout is JSON-RPC only; all logs to stderr; `initialize` waits on no external I/O; startup never panics — all init returns `Result` and degrades gracefully.

## Testing Decisions

**Test philosophy: only external behavior, not implementation details.** Few seams, highest possible. Ideal seam count: one network trait + pure functions.

- **One network seam — a `Fetch` trait.** Core logic depends on an interface that performs HTTP fetches; production wires a real HTTP client, tests inject a fake returning fixed responses. This single seam deterministically tests: fan-out scheduling (first-success-wins, cancel rest), batched retry across instances, health-score promotion/demotion, instance filtering and ranking by latency, result merging and dedup.
- **Pure-function tests (no seam):** `extract(html, word_limit) -> text` (readabilityrs + truncation), and `derive_site_name(url)` / `derive_favicon(url)` (ADR-0005 URL derivation).
- **MCP transport: one lightweight integration test** — the binary starts, responds to `initialize`, and keeps stdout clean. Not behavior-testing the protocol library itself.
- No prior art in this repo (greenfield). The `Fetch` trait is a new seam, introduced at the highest point (the function boundary the whole pipeline funnels through), so one seam covers the bulk of logic.

## Out of Scope

- **Web fetch / reader tool.** This repo is search-only. A separate fetch tool (equivalent to `web_reader`) is a future, separate project.
- **Any summarization** (LLM, extractive, or abstractive). We return raw page-body text only.
- **Self-scraping search engines** (DDG HTML, etc.). Maintenance burden rejected in ADR-0001.
- **Self-hosting SearXNG** or any Docker/Python dependency.
- **Brave / Playwright / any non-SearXNG fallback.** Rejected in ADR-0001.
- **Image / news / video search.** Only `general` (web) results, matching 99% of agent needs.
- **`max_results` parameter.** Result count fixed (~10), not exposed — matches the target tool.
- **Multiple tool exposure.** Exactly one tool: `web_search_prime`.
- **Search-result quality benchmarking vs. the paid tool.** Architecture cannot guarantee parity; verified empirically after build, not part of this spec.

## Further Notes

- **Why not the highest-downloaded `readability` crate?** It ports the 2009 arc90 algorithm, not Mozilla's modern Readability, and is stale. `readabilityrs` is the modern, maintained port. See ADR-0003.
- **Known, accepted trade-off:** search latency and result-quality parity vs. the hosted paid tool cannot be guaranteed by design — they depend on which engines the SearXNG instances expose. This is acceptable for a free, unlimited tool and is to be assessed empirically post-build, not treated as a bug.
- **Decision trail:** all architectural decisions are recorded in `docs/adr/0001`–`0005`; domain vocabulary in `CONTEXT.md`. These are the source of truth this spec synthesizes.
