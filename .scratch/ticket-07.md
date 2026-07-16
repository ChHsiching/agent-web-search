## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

Replace the stub `web_search_prime` handler (from the MCP shell ticket) with a call into the full search pipeline. When the agent calls `tools/call` for `web_search_prime`, the server deserializes the five parameters, runs the search orchestration, and returns the results as a JSON-RPC response. This is the end-to-end connection that makes the tool actually usable by an agent.

Includes a lightweight integration test that exercises the real binary: it starts the server, performs the `initialize` handshake, lists tools, calls `web_search_prime` with a real query, and confirms results come back — while verifying stdout stays clean (JSON-RPC only) throughout. This is the ticket that proves the whole thing works as one piece.

## Acceptance criteria

- [ ] `tools/call` for `web_search_prime` deserializes the five parameters and invokes the search pipeline.
- [ ] A real query returns a JSON-RPC response containing result objects (`title`, `url`, `summary`, `site_name`, `favicon`).
- [ ] Invalid parameters return a clear MCP error, not a crash.
- [ ] A search that finds no available source returns a clear error in the response, not a crash or hang.
- [ ] Integration test: start binary → `initialize` → `tools/list` → `tools/call` with a real query → assert results present → assert stdout across the whole sequence is valid JSON-RPC (no stray log lines).
- [ ] stderr contains the diagnostic logs; stdout does not.

## Blocked by

- #7 — search orchestration: fanout + parse + extract integration (the pipeline this ticket wires to the MCP handler).
