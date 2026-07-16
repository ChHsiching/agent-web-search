## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

Given a query and the ranked instance list (from sources), send the query concurrently to the top 3 instances, take the first successful JSON response, and cancel the rest. If all 3 fail or time out (per-instance timeout ~3–5s), retry against the next batch of 3 down the ranked list, until a success or the list is exhausted. An in-memory health score tracks consecutive failures per instance so that dead instances are temporarily demoted and not retried as primaries. The health score is not persisted — it resets on restart. See ADR-0001.

Built on the `Fetch` trait introduced by sources, so fan-out scheduling, batched retry, and health-score promotion/demotion are all deterministically testable with a fake fetcher that returns canned responses with controlled timing.

This module is the reliability core: it is what makes "one instance down" a non-event. It produces the raw SearXNG JSON from whichever instance answered; parsing that JSON into results is a downstream concern (the query builder / parser ticket).

## Acceptance criteria

- [ ] `fanout(fetcher, instances, query) -> RawResponse` sends the query concurrently to the top 3 ranked instances and returns the first success.
- [ ] Losing (non-winning) requests are cancelled once a success returns.
- [ ] Per-instance timeout (~3–5s) is enforced; a timed-out instance counts as a failure.
- [ ] On all-3 failure, the next batch of 3 is tried, down the list until success or exhaustion.
- [ ] In-memory health score: an instance with consecutive failures is demoted (moved later in the ranking used for fan-out) so it stops being chosen as a primary; it recovers over time or on a success.
- [ ] When every eligible instance is exhausted, returns a clear "no source available" error (not a panic, not a hang).
- [ ] Unit tests (via the fake `Fetch`) cover: first-success-wins, cancel-on-success, per-instance timeout, batched retry on all-fail, health-score demotion after failures, recovery after a success, total exhaustion error.

## Blocked by

- #5 — sources: fetch + filter + cache SearXNG instances from searx.space (provides the ranked instance list and the `Fetch` trait this module builds on).
