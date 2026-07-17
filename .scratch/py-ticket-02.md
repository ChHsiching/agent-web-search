## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

Connect the `web_search_prime` handler to real DuckDuckGo search via the `ddgs` library. Map the five tool parameters to `ddgs.text()` arguments and return real results (title, url, snippet). This is the backend connection — the ticket that proves the verified-stable backend (`ddgs`, 4/4 in testing) actually works inside the server.

Parameter mapping (target tool → ddgs):
- `search_query` → `keywords`
- `search_domain_filter` → folded into the query via `site:` syntax (ddgs accepts it as part of keywords)
- `search_recency_filter` (oneDay/oneWeek/oneMonth/oneYear/noLimit) → `timelimit` (`d`/`w`/`m`/`y`/None)
- `location` (cn/us) → `region` (`cn-zh`/`us-en`)
- `content_size` → not used here (consumed by the extract ticket downstream)

For now this returns raw results (title/url/snippet only); page-body extraction and output assembly come in the orchestration ticket.

## Acceptance criteria

- [ ] `web_search_prime` handler calls `ddgs.text()` and returns real search results.
- [ ] Each of the 5 parameters maps correctly to ddgs arguments (verified by behavior).
- [ ] `search_domain_filter` restricts results to the given domain.
- [ ] `search_recency_filter` values map: oneDay→d, oneWeek→w, oneMonth→m, oneYear→y, noLimit→None.
- [ ] `location` maps to the ddgs region.
- [ ] A real query returns a JSON array of results with title + url.
- [ ] Errors from ddgs (rate-limit, network) are caught and returned as a clear error object, not a crash.
- [ ] `ddgs` is dependency-injected / mockable so the parameter mapping can be unit-tested without hitting the network.

## Blocked by

- #11 — Python MCP shell responding to `initialize` (the handler this wires up).
