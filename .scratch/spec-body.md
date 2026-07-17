## Problem Statement

Coding agents (Claude Code, Codex, ZCode) need web search to look up docs, APIs, libraries, and error solutions. The official/hosted web-search MCP tools they ship with are **paid** (per-query billing, monthly quotas). A developer who just wants their agent to search the web hits a paywall or a `429 Weekly/Monthly Limit Exhausted`.

The user wants a **free, unlimited, stable, drop-in replacement** for the paid `web_search_prime` MCP tool — same tool name, same parameters — so switching is a config change, not a code change.

## Solution

A **single-file stdio MCP server**, written in **Python** (distributed as a **PyInstaller-bundled executable**), that exposes one tool — `web_search_prime` — backed by **DuckDuckGo via the `ddgs` library** (the only empirically-verified-stable free search backend).

- The server is installed by downloading one pre-compiled binary (PyInstaller packs the Python interpreter + code + `ddgs`); **no Docker, no Python install, no API key, no runtime fee.**
- Reliability comes from `ddgs`'s community-maintained anti-bot/rate-limit handling, plus our orchestration layer (top-N page-body extraction, graceful degradation).
- Results include **page-body Extract** (raw text, truncated by `content_size`) for the top results — the agent reads and interprets it; we do **no summarization**.
- The MCP connection **never fails to establish** at startup: measured PyInstaller startup→`initialize` response is ~1.3s, well within agent handshake timeouts; stdout carries only JSON-RPC.

### Why this stack (ADR-0006, empirically verified)

- **SearXNG public instances** (originally chosen in ADR-0001): **0/38 healthy instances return JSON results** — abandoned as infeasible.
- **DuckDuckGo via `ddgs`**: **4/4 queries stable** in testing. The only verified-stable free backend.
- **Rust was tried first** (ADR-0002, 8 tickets built): pipeline correct but no stable DDG backend exists for Rust. Superseded.
- **PyInstaller feasibility**: single-file exe, 0.89s startup (probe) / 1.27s (with ddgs), ~25MB. ddgs search verified working through the bundle.

## User Stories

1. As a developer using Claude Code, I want to replace my paid `web_search_prime` MCP with a free one by changing one config line, so that I don't pay per search.
2. As a developer using Codex, I want the replacement to expose the same tool name and parameters, so that my agent's prompts and tool calls work unchanged.
3. As a developer using ZCode, I want the same drop-in replacement, so that all three agents I use share one free tool.
4. As a developer on any machine, I want the tool to work on Windows, Linux, and macOS, so that I can use it regardless of OS.
5. As a developer, I want to install by downloading a single binary, so that I don't need Docker, Python, or any runtime to use it.
6. As a developer, I want the tool to start in ~1s, so that my agent connects without a cold-start timeout.
7. As a developer, I want the MCP connection to never fail at startup, so that my agent reliably discovers the tool.
8. As a developer, I want searches to be reliable and not rate-limited away, so that the tool is actually usable for daily work.
9. As a developer, I want no API key or registration required, so that setup is frictionless.
10. As a developer, I want to limit results to a specific domain (`search_domain_filter`), so that I can scope a search to one site.
11. As a developer, I want to filter by recency (`search_recency_filter`), so that I can find recent releases or breaking changes.
12. As a developer, I want a region parameter (`location`), so that results match my locale (cn/us).
13. As a developer, I want richer page content when I ask for it (`content_size=high`), so that I get enough context to judge a result without a separate fetch.
14. As an agent, I want the `summary` field to contain real page text, so that I can read and reason about it directly (not a downgraded digest).
15. As an agent, I want each result to include a title, url, site name, and favicon, so that the result schema matches `web_search_prime`.
16. As a maintainer, I want the core logic testable without touching the network, so that I can verify orchestration and parsing deterministically.
17. As a maintainer, I want stdout reserved for JSON-RPC, so that stray logging never corrupts the MCP frame stream.
18. As a maintainer, I want a clear error returned when search fails, so that a query failure is distinguishable from a connection failure.
19. As a developer, I want a clear, simple installation doc, so that I can set it up in under a minute.

## Implementation Decisions

- **Language: Python.** Supersedes ADR-0002's Rust. Chosen because the only verified-stable search backend (`ddgs`) is Python. See ADR-0006.
- **Search source: DuckDuckGo via the `ddgs` library.** Amends ADR-0001 — SearXNG abandoned (0/38 usable). `ddgs` handles anti-bot, rate-limit, and retry logic; community-maintained.
- **Tool interface: 1:1 with `web_search_prime`.** Name `web_search_prime`; parameters `search_query` (required), `search_domain_filter`, `search_recency_filter` (oneDay|oneWeek|oneMonth|oneYear|noLimit), `content_size` (medium|high), `location` (cn|us). Mapped to `ddgs.text()` arguments (region, timelimit, site filter).
- **No summarization.** `content_size` controls how many words of page body to return, not a generated summary. The agent reads raw text. See ADR-0001 scope.
- **Extract applied to top results only** (default 3, hardcoded, not exposed). Remaining results return just the Snippet from the source. Page-body fetched via HTTP + Readability extraction.
- **Output struct per result:** `{ title, url, summary (the Extract), site_name, favicon }`. `site_name` and `favicon` derived from the URL (zero external dependency). See ADR-0005.
- **Content extraction:** a Python Readability port (`readability-lxml` or equivalent) for HTML→main-text. Carries over ADR-0003's intent.
- **Distribution: PyInstaller single-file executable.** Packs Python interpreter + code + `ddgs` into one standalone binary per platform. Built per-target-platform in CI (PyInstaller does not cross-compile). GitHub Releases for Windows/Linux/macOS. See ADR-0006.
- **MCP startup discipline (ADR-0004, re-verified for Python):** stdout is JSON-RPC only (logging to stderr); `initialize` waits on no network; startup never crashes — measured ~1.3s startup in the bundled exe.

## Testing Decisions

**Test philosophy: only external behavior, not implementation details.** Few seams, highest possible.

- **The MCP tool handler is the primary seam.** Test that `web_search_prime` with given params produces the expected result shape (with `ddgs` and the page-fetcher mocked/injected). This covers parameter mapping, orchestration, extract application, and output assembly.
- **Pure-function tests (no network):** URL derivation (`site_name`, `favicon`), content_size→word-limit mapping, result-assembly logic.
- **MCP transport: one integration test** against the real binary (or the unbundled script): start, `initialize`, `tools/list`, verify stdout is clean JSON-RPC.
- `ddgs` and HTTP fetching are dependency-injected so tests don't hit the live network.

## Out of Scope

- **Web fetch / reader tool.** Search-only. A separate fetch tool is a future project.
- **Any summarization** (LLM, extractive, or abstractive). Raw page-body text only.
- **SearXNG / Brave / Playwright / any non-DDG backend.** Rejected (SearXNG 0/38; Brave quality/quote rejected; Playwright heavy).
- **Image / news / video search.** Only `general` (web) results.
- **`max_results` parameter.** Fixed count (~10), not exposed.
- **Multiple tool exposure.** Exactly one tool: `web_search_prime`.

## Further Notes

- **Decision trail:** ADRs 0001–0006 in `docs/adr/`; domain vocabulary in `CONTEXT.md`. ADR-0001 (scope, extract-not-summary), 0003 (Readability), 0004 (MCP startup), 0005 (output schema) carry over; ADR-0002 (Rust) superseded; ADR-0006 records the switch to Python+ddgs+PyInstaller.
- **The Rust implementation was built first** (issues #2–#9) and remains in git history as a design reference; the active codebase is now Python.
