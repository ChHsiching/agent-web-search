# agent-web-search

A **free, unlimited, stable** web-search MCP tool for coding agents (Claude Code, Codex, ZCode). A drop-in replacement for paid/hosted `web_search_prime` — same tool name, same parameters, zero runtime cost.

It searches the web by querying **SearXNG public instances** (rotating across many), with no API key, no Docker, no Python, and no per-query billing. Results include page-body extracts so the agent can read and reason about each hit.

## Why

The official `web_search_prime` tools are metered and return `429 Weekly/Monthly Limit Exhausted` mid-session. This project exists so a developer can give their agent web search that never runs out.

## Install

**Option A — pre-compiled binary (recommended, no toolchain needed):**

1. Download the binary for your platform from [Releases](../../releases):
   - Windows: `agent-web-search-x86_64-pc-windows-msvc.zip`
   - Linux x64: `agent-web-search-x86_64-unknown-linux-gnu.tar.gz`
   - Linux arm64: `agent-web-search-aarch64-unknown-linux-gnu.tar.gz`
   - macOS x64: `agent-web-search-x86_64-apple-darwin.tar.gz`
   - macOS arm64 (Apple Silicon): `agent-web-search-aarch64-apple-darwin.tar.gz`
2. Extract and place the binary on your `PATH`.
3. Configure your agent (below).

**Option B — `cargo install` (needs a Rust toolchain):**

```sh
cargo install agent-web-search
```

## Configure your agent

Add the server to your agent's MCP config as a stdio server running the binary.

**Claude Code / ZCode (`.claude.json` or equivalent):**

```json
{
  "mcpServers": {
    "web-search-prime": {
      "type": "stdio",
      "command": "agent-web-search"
    }
  }
}
```

**Codex:** add the same stdio entry under your MCP servers config.

Once configured, the agent sees a `web_search_prime` tool with the same parameters as the paid version — your prompts and tool calls work unchanged.

## How it works

- **Sources:** queries are fanned out concurrently to the top 3 healthiest SearXNG instances (drawn live from [searx.space](https://searx.space), auto-filtered and ranked by latency). On failure, it retries the next batch down the list. No single instance is a dependency.
- **No key, no fee:** SearXNG public instances are free and volunteer-run; the instance list self-updates, so there is nothing to maintain.
- **Stability first:** the MCP `initialize` handshake waits on no network, stdout carries only JSON-RPC, and the instance list is cached locally so a cold start degrades gracefully.
- **Results:** each result carries `title`, `url`, `summary` (page-body extract for the top 3, source snippet for the rest), `site_name`, and `favicon`. The agent reads the raw text — we do no summarization.

## Tool parameters

| Parameter | Required | Description |
| --- | --- | --- |
| `search_query` | yes | The search terms. |
| `search_domain_filter` | no | Restrict to a domain, e.g. `docs.rust-lang.org`. |
| `search_recency_filter` | no | `oneDay`, `oneWeek`, `oneMonth`, `oneYear`, `noLimit` (default). |
| `content_size` | no | `medium` (~500 words/extract, default) or `high` (~2500 words). |
| `location` | no | `cn` (default) or `us`. |

## Build from source

```sh
git clone https://github.com/ChHsiching/agent-web-search.git
cd agent-web-search
cargo build --release
```

The binary is at `target/release/agent-web-search` (or `.exe` on Windows).

## Development decisions

Architectural decisions are recorded in [`docs/adr/`](docs/adr/), and the domain glossary in [`CONTEXT.md`](CONTEXT.md). See issue #1 for the full spec.
