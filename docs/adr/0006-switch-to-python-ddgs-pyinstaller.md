# Switch to Python + ddgs + PyInstaller (supersedes ADR-0002, amends ADR-0001)

## Status

Accepted. Supersedes ADR-0002 (implementation language: Rust). Amends ADR-0001 (search source: SearXNG public instances — now abandoned as infeasible).

## Context

ADR-0001 chose SearXNG public instances as the search source; ADR-0002 chose Rust + tokio. The full Rust pipeline was built (8 tickets, 45 tests green) but live verification revealed a decisive failure:

**Probing all 38 healthy SearXNG instances (full browser headers) returned 0 JSON search results** — 15 × 429, the rest 403/non-JSON. Public instance operators broadly disable/restrict the JSON API to prevent abuse. The core feasibility assumption of ADR-0001 does not hold in practice.

Investigation of alternatives (all empirically tested):

| Source | Result |
| --- | --- |
| SearXNG public instances | 0/38 usable |
| DDG Instant Answer API | instant-answers only, 0 SERP results |
| DDG HTML (raw GET) | blocked by anti-bot |
| DDG HTML (raw POST + form) | intermittent, rate-limits fast |
| DDG via Python `ddgs` library | **4/4 queries stable, real results** |
| Rust DDG crates (`duckduckgo`, etc.) | immature / CLI-only, not embeddable as a library |
| Go DDG libraries | most popular is 16★ and stale since 2023 — worse than Rust |

The only empirically-verified-stable search backend is **DuckDuckGo accessed through the Python `ddgs` library** (community-maintained anti-bot/rate-limit handling).

## Decision

- **Language: Python.** Supersedes ADR-0002's Rust choice.
- **Search source: DuckDuckGo via the `ddgs` library.** Amends ADR-0001 — SearXNG public instances are abandoned.
- **Distribution: PyInstaller single-file executable.** Packs the Python interpreter + code + `ddgs` deps into one standalone binary, so **users need no Python install** — satisfying the hard constraint ("users must not install an extra environment").

## Feasibility verification (measured, not assumed)

| Concern | Measurement | Verdict |
| --- | --- | --- |
| PyInstaller single-file startup → MCP `initialize` response | **0.89s** (bare Python: 0.50s) | ✅ well within agent handshake timeouts |
| Bundle size | **15.3 MB** | ✅ acceptable single download |
| `ddgs` real search stability | **4/4 queries** returned full results | ✅ only verified-stable backend |
| Zero extra environment for users | single-file exe, no Python install needed | ✅ |

## Why not keep Rust / switch to Go

- Rust's pipeline was correct, but no stable search backend exists for it (SearXNG dead, Rust DDG crates immature). Keeping Rust means self-writing DDG scraping — the "wheel-reinvention / high-maintenance" path rejected twice.
- Go has no more-mature DDG library than Rust; switching languages solves nothing and discards working code.
- Python is the only ecosystem with a verified-stable search backend (`ddgs`). The only objection (Python runtime dependency) is removed by PyInstaller.

## What carries over (the architectural investment is not wasted)

The design decisions that are backend- and language-independent still hold:
- Single stdio MCP server, one `web_search_prime` tool, 5-param schema 1:1 with target (drop-in).
- MCP startup discipline (ADR-0004): clean stdout, no network before handshake, graceful startup. Re-verified for Python+PyInstaller (0.89s startup).
- No summarization — page-body extract by word count (ADR-0001 scope decision).
- Fan-out / retry / health-score *concepts* apply (ddgs handles DDG-internal retry, but the orchestration shape for top-N extract + URL derivation remains).
- Extract via a Readability port (ADR-0003) — Python uses `readability-lxml` / equivalent.
- Output schema: `{title, url, summary, site_name, favicon}`, favicon/site_name derived zero-dependency (ADR-0005).

## Consequences

- The Rust implementation (`src/`, `Cargo.toml`) is superseded; it remains as a design reference but the active codebase moves to Python.
- Cross-platform distribution requires building per-target-platform in CI (PyInstaller does not cross-compile) — Windows/Linux/macOS runners in GitHub Actions.
- Startup is ~0.9s (vs Rust's ms-range) — acceptable, but must keep the post-handshake async pattern (background work after `initialize`, never before).
