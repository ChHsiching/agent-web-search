## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

Fetch the list of SearXNG instances from `https://searx.space/data/instances.json`, filter to healthy ones (`timing.search.success_percentage == 100 && http.status_code == 200`), detect which expose the JSON API (one-shot probe with `format=json`), rank by search-latency median, and cache the result locally (~1h TTL) so a later startup can degrade to cache if searx.space is unreachable. The list is never hand-maintained. See ADR-0001.

The HTTP layer is abstracted behind a `Fetch` trait — the single test seam for the whole codebase. Production wires a real HTTP client; tests inject a fake returning fixed responses. This trait is introduced here because sources is the first module to touch the network, and it will be reused by fanout (next ticket) and page-body extraction.

Critical timing constraint (ADR-0004): the searx.space fetch runs **after** the MCP handshake, asynchronously in the background. A query arriving with an empty in-memory list triggers one short synchronous fetch for that query only. The handshake is never blocked.

## Acceptance criteria

- [ ] A `Fetch` trait exists abstracting HTTP GET (returns status + body); a real implementation and a test fake both implement it.
- [ ] `fetch_instances(fetcher) -> Vec<Instance>` pulls `instances.json`, filters to healthy instances, and returns them with their metadata (url, latency median, etc.).
- [ ] JSON-API-open detection: each candidate is probed once with `format=json`; usable ones are kept, others dropped.
- [ ] Instances are ranked by search-latency median (fastest first).
- [ ] The fetched list is cached locally (file under the user's cache dir) with a ~1h TTL.
- [ ] On startup with a stale/missing cache and searx.space down, the system does not fail to start — it logs to stderr and proceeds with an empty list.
- [ ] No network call blocks the MCP `initialize` handshake.
- [ ] Unit tests (via the fake `Fetch`) cover: filtering healthy vs unhealthy, JSON-API detection, ranking by latency, cache write/read, graceful behavior on searx.space failure.

## Blocked by

- #2 — Cargo skeleton + MCP shell that responds to `initialize` (introduces the binary the sources module plugs into; establishes the stderr-logging and no-panic conventions).
