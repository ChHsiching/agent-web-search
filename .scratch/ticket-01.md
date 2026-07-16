## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

A runnable Rust binary that acts as a stdio MCP server. On launch it responds to the MCP `initialize` handshake immediately (without touching the network), advertises exactly one tool (`web_search_prime`) via `tools/list`, and accepts `tools/call` for that tool — returning a hardcoded stub response for now. The full search pipeline is wired up in a later ticket; this ticket stands up the protocol shell and, critically, proves the connection is stable.

This is the layer most likely to fail in the wild, so it is built and verified first: stdout carries JSON-RPC frames only (no stray logs), all diagnostics go to stderr, startup never panics, and the handshake waits on no external I/O. See ADR-0004.

## Acceptance criteria

- [ ] `cargo build` produces a single binary.
- [ ] The binary, launched as a stdio MCP server, responds correctly to an `initialize` request (returns protocol version, server info, capabilities).
- [ ] `tools/list` returns exactly one tool named `web_search_prime` with the five-parameter input schema matching the target tool (`search_query` required; `search_domain_filter`, `search_recency_filter`, `content_size`, `location` optional).
- [ ] `tools/call` for `web_search_prime` returns a stub response (e.g. an empty result list or a placeholder) without error.
- [ ] Nothing is written to stdout except valid JSON-RPC frames — verified by capturing stdout and confirming it parses cleanly as JSON-RPC across a handshake + tools/list + tools/call sequence.
- [ ] No network call is made before the `initialize` response is sent.
- [ ] Startup does not panic on any reasonable input (missing config, empty cache, etc.) — degrades gracefully.

## Blocked by

- None — can start immediately.
