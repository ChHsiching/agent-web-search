# agent-web-search

A **free, unlimited, stable** web-search MCP tool for coding agents (Claude Code, Codex, ZCode). A drop-in replacement for paid/hosted `web_search_prime` — same tool name, same parameters, zero runtime cost.

It searches the web via **DuckDuckGo** (using the [`ddgs`](https://github.com/deedy5/duckduckgo_search) library, which handles anti-bot/rate-limit logic), with no API key, no Docker, no per-query billing. Results include page-body extracts so the agent can read and reason about each hit.

## Why

The official `web_search_prime` tools are metered and return `429 Weekly/Monthly Limit Exhausted` mid-session. This project exists so a developer can give their agent web search that never runs out.

## Install

**Option A — pre-compiled binary (recommended, no Python needed):**

1. Download the archive for your platform from [Releases](../../releases):
   - Windows: `agent-web-search-windows-x64.zip`
   - Linux: `agent-web-search-linux-x64.tar.gz`
   - macOS: `agent-web-search-macos.tar.gz`
2. Extract and place the binary on your `PATH`.
3. Configure your agent (below).

The binary is a PyInstaller bundle — Python interpreter and all dependencies are packed inside, so you do **not** need Python installed.

**Option B — from source (needs Python 3.10+):**

```sh
git clone https://github.com/ChHsiching/agent-web-search.git
cd agent-web-search
pip install -e .
```

Then run via `agent-web-search` (the installed script) or `python -m agent_web_search`.

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

- **Search backend:** DuckDuckGo via the `ddgs` library — the only empirically-verified-stable free search backend. `ddgs` handles the anti-bot, rate-limit, and retry logic so we don't have to.
- **No key, no fee:** DuckDuckGo search is free; `ddgs` is open source. Nothing to register, nothing to pay.
- **Stability first:** the MCP `initialize` handshake waits on no network (~1s startup in the bundled binary), stdout carries only JSON-RPC, and errors degrade gracefully.
- **Results:** each result carries `title`, `url`, `summary` (page-body extract for the top 3, source snippet for the rest), `site_name`, and `favicon`. The agent reads the raw text — we do no summarization.

## Tool parameters

| Parameter | Required | Description |
| --- | --- | --- |
| `search_query` | yes | The search terms. |
| `search_domain_filter` | no | Restrict to a domain, e.g. `docs.rust-lang.org`. |
| `search_recency_filter` | no | `oneDay`, `oneWeek`, `oneMonth`, `oneYear`, `noLimit` (default). |
| `content_size` | no | `medium` (~500 words/extract, default) or `high` (~2500 words). |
| `location` | no | `cn` (default) or `us`. |

## Build from source (PyInstaller)

To produce a standalone binary yourself:

```sh
pip install pyinstaller
pip install -e .
pyinstaller agent-web-search.spec --noconfirm
```

The binary is at `dist/agent-web-search` (or `.exe` on Windows).

## Development decisions

Architectural decisions are recorded in [`docs/adr/`](docs/adr/), and the domain glossary in [`CONTEXT.md`](CONTEXT.md). See issue #1 for the full spec. Key note: an earlier version targeted SearXNG public instances in Rust, but live testing found 0/38 instances usable — the project switched to DuckDuckGo via `ddgs` (ADR-0006).
