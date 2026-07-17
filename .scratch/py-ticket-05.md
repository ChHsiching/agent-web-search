## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

Replace the stub `web_search_prime` handler with a call into the full search orchestration. When the agent calls `tools/call`, the server deserializes the five parameters, runs the orchestration, and returns the assembled results as a JSON-RPC response. This is the end-to-end connection that makes the tool usable by an agent.

Includes an integration test that exercises the real (unbundled) server: start, `initialize`, `tools/list`, call `web_search_prime` with a real query, and confirm results come back — while verifying stdout stays clean (JSON-RPC only) throughout.

## Acceptance criteria

- [ ] `tools/call` for `web_search_prime` deserializes parameters and invokes the orchestration.
- [ ] A real query returns a JSON-RPC response with result objects (`title`, `url`, `summary`, `site_name`, `favicon`).
- [ ] Invalid parameters return a clear MCP error, not a crash.
- [ ] A search that fails (ddgs error, empty) returns a clear error in the response, not a crash or hang.
- [ ] Integration test: start server → initialize → tools/list → tools/call real query → assert results present → assert stdout across the whole sequence is valid JSON-RPC.
- [ ] stderr contains diagnostic logs; stdout does not.

## Blocked by

- #14 — search orchestration: ddgs results + top-N extract + output assembly (the pipeline this wires to the handler).
