## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

A Python script acting as a stdio MCP server. On launch it responds to the MCP `initialize` handshake immediately (no network), advertises one tool (`web_search_prime`) via `tools/list` with the 5-parameter schema matching the target tool, and accepts `tools/call` returning a stub for now. The real search pipeline is wired up in a later ticket; this ticket stands up the protocol shell and proves the connection is stable — including when bundled with PyInstaller (measured ~1.3s startup).

This is the layer most likely to fail in the wild, so it is built first: stdout carries JSON-RPC only (all logging to stderr), the handshake waits on no network, and startup never crashes. See ADR-0004.

## Acceptance criteria

- [ ] A Python entry point runs as a stdio MCP server (using the `mcp` Python SDK).
- [ ] Responds correctly to `initialize` (returns server info "agent-web-search", tools capability).
- [ ] `tools/list` returns exactly one tool `web_search_prime` with the 5-param schema: `search_query` (required), `search_domain_filter`, `search_recency_filter`, `content_size`, `location` (all optional strings).
- [ ] `tools/call` returns a stub response without error.
- [ ] Nothing written to stdout except JSON-RPC — verified across an initialize + tools/list + tools/call sequence.
- [ ] All logs go to stderr.
- [ ] No network call before `initialize` responds.
- [ ] (Verified) A PyInstaller single-file bundle of this script starts and responds to `initialize` within ~2s.

## Blocked by

- None — can start immediately.
