# Search source architecture

We need a search backend for a free, unlimited, stable web-search tool. We will query **SearXNG public instances** (rotating across many, via their JSON API). Reliability comes entirely from fanning out across many instances — there is no non-SearXNG fallback. We will **not** self-scrape search engines (e.g. DuckDuckGo HTML), nor self-host SearXNG.

## Considered options

- **Self-scrape engines (e.g. DDG HTML)** — rejected: scraping is inherently fragile; every engine redesign breaks our parser, creating permanent, unbounded maintenance. Reinvents what SearXNG already does.
- **Self-host SearXNG (Docker/app)** — rejected: requires Docker + a Python web app, contradicting the single-binary, low-spec-hardware (Intel 9th-gen-class), zero-dependency deployment goal.
- **SearXNG public instances (JSON API)** — chosen: zero install (we are only an HTTP client), zero engine maintenance (the community/instance operators maintain the engine adapters), free, and effectively unbounded by rotating across the ~43 healthy instances listed at searx.space. Robustness comes from multi-instance fan-out, not from any single instance's reliability.
- **Brave Search API free tier** — rejected as fallback: requires every user to register an API key (installation friction) for a capped 2,000 queries/month, and result quality is worse than driving a headless browser (Playwright) — which itself was rejected below. Not worth the friction or quality cost.
- **Playwright / headless-browser fallback** — rejected: would bundle a ~150MB+ browser engine into the install, directly violating the single-binary, lightweight, low-spec-hardware, zero-dependency goal. Paying a fixed heavy dependency for a tail-risk event.

## Scope: search + content extraction, NOT summarization

The target tool (`web_search_prime`) returns a summary per result controlled by `content_size` (medium ≈ 400–600 words, high ≈ 2500 words). We reproduce the **parameter and the content-fetching** but **not the summarization**:

- The tool **fetches page body** for top results and returns a **word-limited extract of the raw body** (medium ≈ 500 words, high ≈ 2500 words).
- The returned `summary` field is page-body extract, not a generated/abstractive/extractive summary.
- **The agent reads and interprets the raw extract itself** — we deliberately do not pre-digest content with a weaker process before it reaches the LLM that is best at understanding it.

## Considered options for content/summary

- **Abstractive summary via local LLM** — rejected: a 7B-class summarization model needs ~4–5GB RAM and tens of seconds to minutes per result on Intel 9th-gen-class CPUs, directly violating the low-spec, stable, dependency-free goal.
- **Abstractive summary via remote/paid LLM** — rejected: per-query API cost conflicts with the free, unlimited core requirement.
- **Extractive summary (sentence-scoring)** — rejected: still pre-digests content; simpler than LLM but loses information vs. returning raw body extract, for no gain since the agent can read the raw text itself.
- **No summary — raw body extract by word count** — chosen: zero summarization compute, no LLM dependency, deterministic, and higher fidelity (the agent gets original text, not a downgraded digest). `content_size` controls only how many words of page body to return.

## Consequences

- We write only fan-out + JSON-merge + fallback + page-fetch/extract logic; no scrapers to maintain, no summarizer to run.
- Reliability depends on the fallback/rotation layer (for search sources) and on the page-fetch layer (for body extraction), not on any single source.
- A user who wants stronger guarantees may later point the tool at their own (local or private) SearXNG instance as an additional source; this is just another source in the fallback chain, not a new architecture.

## Instance list source + rotation strategy (no maintenance)

**Zero-maintenance requirement:** the maintainers must never need to manually curate or periodically update the instance list.

- **List source:** the instance list is fetched live at startup from `https://searx.space/data/instances.json` (a public, community-maintained registry reporting each instance's search success %, HTTP status, TLS grade, SearXNG version, and search-latency median). We consume it; we do not maintain it.
- **Auto-filtering:** only instances with `timing.search.success_percentage == 100` and `http.status_code == 200` are eligible.
- **API-open detection:** searx.space does not report whether JSON API is enabled. Each candidate is probed once with `format=json`; success marks it usable, failure marks it skipped. This is a one-shot self-check, not a manual step.
- **Ranking:** eligible instances are ranked by search-latency median (fastest first); the data is provided by searx.space.
- **Rotation:** a query is fanned out concurrently to the top 3 ranked instances; the first successful response wins and the rest are cancelled.
- **Health score:** an in-memory score tracks consecutive failures per instance; failing instances are temporarily demoted so dead instances are not retried as primaries. Not persisted — resets on restart.
- **Fallback:** if all 3 fanned-out instances fail or time out, the query is retried against the next batch of 3 ranked instances, and so on down the eligible list (~43 healthy instances ≈ 14 batches). The query only fails if every eligible instance has been exhausted. This is stronger than a capped API fallback (43 free instances vs 2,000/month) and needs no API key or registration.
- **Cache + degrade:** the fetched instance list is cached locally (e.g. ~1h TTL). If searx.space itself is unreachable on a later startup, the tool falls back to the last cached list rather than failing to start. The cache auto-refreshes whenever searx.space is reachable again.
- **Per-instance timeout:** short (~3–5s) so a dead instance is abandoned quickly, not waited on.

This delivers "latest + stable + zero-maintenance" simultaneously: the list is always as fresh as searx.space, fan-out + health-score + batched retry across ~43 instances carry reliability, and no human ever needs to edit an instance list or register any API key.
