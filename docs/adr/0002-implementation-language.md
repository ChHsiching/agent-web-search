# Implementation language: Rust

The tool will be written in **Rust**, with **tokio** for async I/O and an existing Rust MCP SDK for the protocol layer (no hand-rolled stdio framing).

## Considered options

- **Go** — rejected: its single-binary, fast-startup, and strong concurrency story match our needs, but its error handling (`if err != nil`) is easy to silently skip and its concurrency safety is runtime-checked, not compile-time-enforced. For a tool whose primary requirement is stability and whose surface is dominated by concurrent fan-out + error handling, we prefer the guarantees Rust's type system gives at compile time.
- **Node/TypeScript** — rejected: `npx` cold-start is the leading cause of MCP connection failures we explicitly want to eliminate; selecting it would write the root cause into the prescription.
- **Python** — rejected: cold-start + runtime dependency management, same MCP reliability concern, worse than Node here.

## Consequences

- Concurrency (fan-out across multiple SearXNG instances + Brave) is async via tokio; `Send`/`Sync` and ownership catch data races at compile time.
- Error handling uses `Result` + `?`; an unhandled error is a compile error, not a silent skip — directly serving the "stability from the ground up" goal.
- The project is expected to stay small (fan-out + JSON merge + MCP shell), so Rust's slower development pace is an acceptable trade for the stronger guarantees.
- Cross-compilation still yields a single static binary per OS/arch, so distribution and startup are on par with Go.
