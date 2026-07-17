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
2. Extract the archive — you get a single `agent-web-search` (or `.exe`) binary.
3. Note its full path, e.g. `C:\Tools\agent-web-search.exe` or `/home/you/bin/agent-web-search`.
4. Configure your agent using that path (below).

The binary is a PyInstaller bundle — the Python interpreter and all dependencies are packed inside, so you do **not** need Python installed. (You may also place the binary on your `PATH` and use just the name `agent-web-search` as the command.)

**Option B — from source (needs Python 3.10+):**

```sh
git clone https://github.com/ChHsiching/agent-web-search.git
cd agent-web-search
pip install -e .
```

This installs an `agent-web-search` script on your `PATH`. The command below then uses just the name.

## Configure your agent

This is a standard stdio MCP server. Add it to your agent's MCP config — the shape below works for **ZCode, Claude Code, and Codex** (each reads the same `mcpServers` block):

```json
{
  "mcpServers": {
    "agent-web-search": {
      "type": "stdio",
      "command": "/absolute/path/to/agent-web-search",
      "args": []
    }
  }
}
```

- **`command`**: the full path to the binary you extracted (Option A), or just `agent-web-search` if it's on your `PATH` (Option B).
- **`args`**: empty — the server takes no arguments.
- **`type`**: must be `"stdio"`.

The server key (here `agent-web-search`) is your label — name it whatever you like. It does **not** need to match the official `web-search-prime`.

> ⚠️ **Don't reuse the key `web-search-prime`** if you still have the official one configured — the keys would collide and one would silently overwrite the other. Use a distinct key like `agent-web-search`. To fully *replace* the official tool instead, first remove/rename its entry, then you may reuse `web-search-prime`.

The server exposes a tool named **`web_search_prime`** — the same tool name and parameters as the paid version. So your agent's prompts and tool calls work unchanged regardless of the server key you chose.

> Note: if two loaded servers both expose a tool named `web_search_prime`, agent behavior is undefined (one usually shadows the other). Keep only one of them configured to avoid ambiguity.

**Per-agent config file locations** (where to put the block above):

- **ZCode**: your ZCode MCP config (see ZCode docs for the exact file).
- **Claude Code**: `~/.claude.json` (or the project `.mcp.json`).
- **Codex**: your Codex MCP servers config.

Once configured, restart the agent. It will discover the `web_search_prime` tool.

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
