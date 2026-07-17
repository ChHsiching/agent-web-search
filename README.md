# agent-web-search

**English** | [简体中文](./README.zh-CN.md)

A **free, uncapped web search** for MCP-capable agents (Claude Code, ZCode, and
any client that speaks the Model Context Protocol). It runs a search query and
returns results with page-body extracts the agent can read directly — a
drop-in replacement for the paid, rate-limited `web_search_prime` tool built
into many agent clients.

- **Free and uncapped** — searches DuckDuckGo over plain HTTP. No API key, no
  quota, no monthly limit, no 429s.
- **Drop-in compatible** — exposes a tool named `web_search_prime` with the
  same name and parameters as the paid one, so your agent's prompts and tool
  calls work unchanged.
- **Reliable launch** — ships as a single self-contained binary (Python
  interpreter bundled inside). No `npx`, no runtime to install, no network at
  startup. It connects the first time, every time.
- **Near-zero maintenance** — DuckDuckGo access goes through the `ddgs`
  library, which handles anti-bot, rate-limit, and retry logic for us. No
  scraping engine to keep up with.

## What it does

Give it a search query, get back a ranked list of results with page-body
extracts:

```
web_search_prime({ "search_query": "rust tokio tutorial" })
  → [{ "title": "Tutorial | Tokio", "url": "https://tokio.rs/tokio/tutorial",
       "summary": "Tokio is an asynchronous runtime for the Rust …",
       "site_name": "tokio.rs", "favicon": "https://tokio.rs/favicon.ico" },
     …]
```

The top results get their page body fetched and extracted (via a Readability
engine), so the agent can read the actual content rather than just a short
snippet. Results beyond the top few carry the source's own snippet. We return
content, not summaries — no model calls, no digests.

### Tool parameters

| Parameter                | Required | Default  | Description                                                       |
| ------------------------ | -------- | -------- | ----------------------------------------------------------------- |
| `search_query`           | yes      | —        | The search terms.                                                 |
| `search_domain_filter`   | no       | —        | Restrict to a domain, e.g. `docs.rust-lang.org`.                  |
| `search_recency_filter`  | no       | `noLimit`| `oneDay`, `oneWeek`, `oneMonth`, `oneYear`, `noLimit`.            |
| `content_size`           | no       | `medium` | `medium` (~500-word extract) or `high` (~2500-word extract).      |
| `location`               | no       | `cn`     | `cn` or `us`.                                                     |

### What it does *not* do

- **No web fetch / reader** — it searches; it does not fetch arbitrary URLs.
  (That's a separate tool — see [`agent-web-fetch`](https://github.com/ChHsiching/agent-web-fetch).)
- **No summarization / translation** — it returns page-body text, it doesn't
  process it. No paid model calls.
- **No image / news / video search** — only general web results, which covers
  the vast majority of agent needs.
- **No `max_results`** — the result count is fixed (~10) and not exposed,
  matching the target tool.

## Install

### 1. Download the binary for your platform

Grab the right archive from the [latest release](../../releases):

| Platform | File |
| --- | --- |
| Windows | `agent-web-search-windows-x64.zip` |
| Linux | `agent-web-search-linux-x64.tar.gz` |
| macOS | `agent-web-search-macos.tar.gz` |

Extract it to get a single `agent-web-search` (or `.exe`) binary. No installer,
no runtime to install (no Node or Python required — the Python interpreter is
packed inside the binary).

**Where to put the file:** a recommended per-user location exists on each
platform — it needs no admin rights and is the conventional spot for
user-installed programs:

| Platform | Recommended location |
| --- | --- |
| Windows | `%LOCALAPPDATA%\Programs\agent-web-search\agent-web-search.exe` |
| macOS / Linux | `~/.local/bin/agent-web-search` |

That said, MCP doesn't actually care where the file lives — it launches the
binary via the absolute path in your config, so you can put it anywhere you
have read/execute permission (no need to add it to `PATH`). Just don't drop it
into other users' directories or system folders that need admin rights. The
examples below use the recommended locations; replace the path if you put it
elsewhere.

### 2. Register it with your MCP client

This is a standard **stdio MCP server**: it has no arguments and no environment
requirements. In every MCP client the config entry is the same idea — point
`command` at the binary's absolute path, leave `args` empty:

```json
"chhsich-web-search": {
  "type": "stdio",
  "command": "/absolute/path/to/agent-web-search",
  "args": []
}
```

What differs between clients is only **where** this entry goes and the exact
key names. Concrete examples for the common ones:

**ZCode** — add the entry to its MCP servers config (a flat object keyed by
server name, no outer wrapper):

```json
{
  "chhsich-web-search": {
    "type": "stdio",
    "command": "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe",
    "args": []
  }
}
```

**Claude Code** — `~/.claude.json` (or `%USERPROFILE%\.claude.json` on Windows),
where servers live under a `mcpServers` key:

```json
{
  "mcpServers": {
    "chhsich-web-search": {
      "type": "stdio",
      "command": "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe",
      "args": []
    }
  }
}
```

Or via the CLI (does the same thing): `claude mcp add chhsich-web-search "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe"`

**Any other stdio MCP client** — find where it keeps its MCP server list (a
JSON/YAML config, a settings UI, etc.) and add one entry: type `stdio`,
`command` = absolute path to the binary, `args` = `[]`. That's the whole
contract — there are no other parameters to set.

> **Replace the path:** the examples above use the recommended install location
> with `<username>` as a placeholder — swap in your actual username (or use
> `%LOCALAPPDATA%` if your client expands env vars). Adjust the path if you put
> the binary somewhere else.

> **Naming:** the key (`chhsich-web-search` above) is your client-side label
> for the server — call it whatever you want, but use a distinct key so it
> doesn't collide with the official `web-search-prime` entry if you still have
> it. The tool it exposes is named `web_search_prime` (deliberately, for
> drop-in compatibility): two servers that both expose a tool named
> `web_search_prime` are ambiguous, so keep only one configured. To fully
> *replace* the official tool, remove/rename its entry first, then you may
> reuse the key `web-search-prime`.

> **Path tip (Windows):** use the full absolute path including `.exe`.
> Forward slashes work in JSON and avoid backslash escaping.

> **Windows SmartScreen note:** the release binary is unsigned, so Windows may
> show a "Windows protected your PC" prompt the first time it runs. Click
> **More info → Run anyway**. This is expected for unsigned binaries and only
> happens once.

Restart your client after editing the config. The `web_search_prime` tool now
appears alongside the built-in tools and the model can call it like any other.

### 3. Verify it works

After restarting your client, ask the model to search for anything, e.g.:

> Use the web_search_prime tool to search for "rust async runtime"

You should get back a list of results, each with a title, URL, summary,
site name, and favicon. If the tool is missing or returns nothing, check that
the `command` path points at the binary you extracted.

## Build from source

Requires Python 3.10+.

```sh
# Install in editable mode (dev dependencies included)
pip install -e .

# Produce a standalone binary via PyInstaller
pip install pyinstaller
pyinstaller agent-web-search.spec --noconfirm
# → dist/agent-web-search (or .exe on Windows)

# Run the test suite
python -m pytest
```

Each release binary is a PyInstaller bundle — the Python interpreter and all
dependencies packed inside one file.

## How it works

```
query → map params → DuckDuckGo (ddgs: anti-bot/retry handled)
     → result list (title/url/snippet per hit)
     → top results: fetch page → Readability extract → word-limited body text
                                                        ↘ snippet fallback (never empty)
     → assemble {title, url, summary, site_name, favicon} → JSON
```

Search and page-fetch go through dependency-injected seams, so the core logic
fan-out, extraction, and assembly are unit-tested without touching the network.
Every failure (rate-limit, empty results, page-fetch error) comes back as a
structured response the model can read — the server process never crashes.

See `CONTEXT.md` for the project glossary and `docs/adr/` for the architectural
decisions. Note: an earlier version targeted SearXNG public instances (in
Rust), but live testing found 0/38 instances usable — the project switched to
DuckDuckGo via `ddgs` (ADR-0006).
