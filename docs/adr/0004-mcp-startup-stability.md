# MCP startup stability discipline

The single hardest requirement from the user: **the MCP connection must never fail to establish at startup.** The leading causes of MCP connection failures in the wild are implementation defects, not protocol limitations. This ADR records the disciplines that keep our server immune to them, so they are not accidentally broken later.

## Considered context

Observed failure modes of stdio MCP servers in the wild:
- `npx` cold-start (Node only) — the server's first launch triggers a package download; the agent times out before the server is ready. **N/A to us** (Rust static binary), but recorded so no one later moves to a Node-based launcher.
- **stdout pollution** — the server prints a stray `log`/`println` to stdout, corrupting the JSON-RPC frame stream; the agent sees malformed frames and drops the connection. The #1 real-world cause.
- **Blocking startup** — the server does heavy/synchronous work (network calls, file I/O) before replying to the MCP `initialize` handshake; the agent times out and reports a connection failure.
- **Startup panic** — an unwrap/init failure crashes the process during handshake.

## Decisions (non-negotiable)

1. **stdout is JSON-RPC only.** All logs, diagnostics, and errors go to **stderr**. No `println!`/`print!`/`dbg!` to stdout anywhere in the codebase. The MCP SDK's stdio transport owns stdout exclusively.
2. **The `initialize` handshake waits on nothing external.** No network calls, no file reads beyond local config, before the server replies to `initialize`. The instance-list fetch from searx.space happens **after** handshake, asynchronously (see below).
3. **Startup never panics.** All initialization paths return `Result`; a failure degrades gracefully (empty instance list, logged to stderr) rather than crashing the process. The server stays alive and responsive even in a degraded state.
4. **Fast, deterministic startup.** A Rust static binary starts in milliseconds; we keep it that way by deferring all I/O-bound work to post-handshake background tasks.

## Instance-list fetch timing (amends ADR-0001)

ADR-0001 said the instance list is "fetched live at startup." This is refined to protect the handshake:

- **On startup:** reply to `initialize` immediately using whatever is in the local cache (possibly empty on first-ever run). **Do not** block on searx.space.
- **Post-handshake (background):** fetch from searx.space, update the in-memory list, write the local cache.
- **On a query when the in-memory list is empty:** perform one short-timeout synchronous fetch for that query only. If it fails, return a clear "no instances available" error — a **query** failure, never a **connection** failure.

This means searx.space availability affects query freshness, never connection stability. No baked-in instance list is needed (avoids the maintenance burden rejected in ADR-0001); the worst case is a delayed first query on a cold start with searx.space down, not a failed connection.

## Consequences

- The MCP connection is structurally reliable: it cannot be broken by searx.space being down, by slow networks, or by stray log output.
- Any future change that adds synchronous network/file work before `initialize`, or that writes to stdout, must be rejected in review — these are the lines that, once crossed, reintroduce the exact failures this tool exists to avoid.
