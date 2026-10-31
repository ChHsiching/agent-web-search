# agent-web-search

[![GitHub release](https://img.shields.io/github/v/tag/ChHsiching/agent-web-search?include_prereleases&sort=semver)](https://github.com/ChHsiching/agent-web-search/releases/latest)
[![License](https://img.shields.io/github/license/ChHsiching/agent-web-search)](https://github.com/ChHsiching/agent-web-search/blob/main/LICENSE)

**English** | [简体中文](./README.zh-CN.md)

A **free, uncapped web search** for MCP-capable agents. It works with Claude
Code, ZCode, Zed, opencode, and any client that speaks the Model Context
Protocol. It runs a search query and returns results with page-body extracts
the agent can read directly, with no API key, quota, or monthly limit.

## Why this instead of a paid search API

- **Free and uncapped.** Searches DuckDuckGo over plain HTTP. No API key, no
  quota, no monthly limit, no 429s.
- **Universal MCP tool.** Exposes a tool named `web_search` with the standard
  schema agents already know: query, domain filter, recency, content size,
  location. It works in any MCP client without prompt changes.
- **Reliable launch.** Ships as a single self-contained binary with the Python
  interpreter bundled inside. No `npx`, no runtime to install, no network at
  startup.
- **Near-zero maintenance.** DuckDuckGo access goes through the `ddgs`
  library, which handles anti-bot, rate-limit, and retry logic. No scraping
  engine to keep up with.

Hosted search APIs route every query through a vendor cloud, hand you an API
key to guard, and meter each request against a quota you pay to raise. This
tool does the same job from your own machine, for $0 per query. It talks to
DuckDuckGo over plain HTTP, in a process your MCP client already manages.

## What it does

Give it a search query, get back a ranked list of results with page-body
extracts:

```
web_search({ "search_query": "rust tokio tutorial" })
  → [{ "title": "Tutorial | Tokio", "url": "https://tokio.rs/tokio/tutorial",
       "summary": "Tokio is an asynchronous runtime for the Rust …",
       "site_name": "tokio.rs", "favicon": "https://tokio.rs/favicon.ico" },
     …]
```

The top results get their page body fetched and run through a Readability
engine, so the agent reads the actual content rather than a short snippet.
Results beyond the top few carry the source's own snippet. We return content,
not summaries. No model calls, no digests.

### Tool parameters

| Parameter                | Required | Default  | Description                                                       |
| ------------------------ | -------- | -------- | ----------------------------------------------------------------- |
| `search_query`           | yes      | none     | The search terms.                                                 |
| `search_domain_filter`   | no       | none     | Restrict to a domain such as `docs.rust-lang.org`.                |
| `search_recency_filter`  | no       | `noLimit`| `oneDay`, `oneWeek`, `oneMonth`, `oneYear`, `noLimit`.            |
| `content_size`           | no       | `medium` | `medium` gives about 500 words of extract, `high` about 2500.     |
| `location`               | no       | `cn`     | `cn` or `us`.                                                     |

### What it does *not* do

- **No web fetch / reader.** It searches, it does not fetch arbitrary URLs.
  Fetching pages is a separate tool, [`agent-web-fetch`](https://github.com/ChHsiching/agent-web-fetch).
- **No summarization / translation.** It returns page-body text, it doesn't
  process it. No paid model calls.
- **No image / news / video search.** Only general web results, which covers
  the vast majority of agent needs.
- **No `max_results`.** The tool requests 10 results and upstream may return
  fewer; the count is not exposed, matching the target tool.

### Known limitations

These are inherent to the free, dependency-light design, not bugs:

- **Recency filter is best-effort.** `search_recency_filter` is forwarded to
  the upstream Source, which applies it loosely. Results are mostly recent but
  not strictly bounded. This is upstream behavior we cannot tighten without
  taking on a heavier-weight Source.
- **No published-date field.** Each result carries title, url, summary,
  site_name, and favicon only. The upstream Source returns no timestamp, so
  there is nothing to expose. This is the scope of the underlying data, not a
  missing feature.
- **Quality depends on query form.** Keyword queries such as
  `rust tokio spawn example` return sharply more relevant results than full
  natural-language questions, especially for Chinese queries. The tool
  description already asks the model for keywords; a prompt that encourages
  keyword queries helps further.
- **No event clustering / structured digest.** Out of scope. The tool returns
  a flat ranked list with page-body extracts.

## Quick start

### 1. Install the binary for your platform

Grab the right file from the [latest release](../../releases). It's a single,
self-contained binary. The Python interpreter and all dependencies are packed
inside, so there is no archive and no extraction step:

| Platform | File |
| --- | --- |
| Windows | `agent-web-search-windows-x64.exe` |
| Linux | `agent-web-search-linux-x64` |
| macOS | `agent-web-search-macos` |

No installer and no runtime to install. You don't need Node or Python.

**Where to put the file.** A recommended per-user location exists on each
platform. It needs no admin rights and is the conventional spot for
user-installed programs:

| Platform | Recommended location |
| --- | --- |
| Windows | `%LOCALAPPDATA%\Programs\agent-web-search\agent-web-search.exe` |
| macOS / Linux | `~/.local/bin/agent-web-search` |

That said, MCP doesn't care where the file lives. It launches the binary via
the absolute path in your config, so any location you can read and execute
works, and you don't need to add it to `PATH`. Just don't put it in other
users' directories or system folders that need admin rights. The examples
below use the recommended locations; replace the path if you put it elsewhere.

<details>
<summary>Build from source instead, requires Python 3.10+</summary>

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

Each release binary is a PyInstaller bundle, with the Python interpreter and
all dependencies packed inside one file.

</details>

### 2. Register it with your MCP client

This is a standard **stdio MCP server** with no arguments and no environment
requirements. **You never launch it yourself.** Running the binary by hand
just blocks waiting on stdin. Your MCP client starts and stops the server on
demand. The [MCP transports
documentation](https://modelcontextprotocol.io/docs/concepts/transports)
explains how stdio servers work. All you do is point your client's config at
the binary.

In every MCP client the entry looks the same. Point `command` at the binary's
absolute path and leave `args` empty:

```json
"chhsich-web-search": {
  "type": "stdio",
  "command": "/absolute/path/to/agent-web-search",
  "args": []
}
```

What differs between clients is only **where** this entry goes and the exact
key names. Concrete examples for the common ones:

<details>
<summary><strong>ZCode:</strong> flat object keyed by server name</summary>

Add the entry to its MCP servers config, a flat object keyed by server name
with no outer wrapper:

```json
{
  "chhsich-web-search": {
    "type": "stdio",
    "command": "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe",
    "args": []
  }
}
```
</details>

<details>
<summary><strong>Claude Code:</strong> <code>~/.claude.json</code> under a <code>mcpServers</code> key</summary>

Servers live under a `mcpServers` key in `~/.claude.json`, which is
`%USERPROFILE%\.claude.json` on Windows:

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

Or add it from the CLI with `claude mcp add chhsich-web-search "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe"`.
</details>

<details>
<summary><strong>Zed:</strong> <code>context_servers</code> key, no <code>type</code> field</summary>

Easiest through the UI: **Settings → AI → MCP Servers → Add Server → Add Local
Server**, pointing it at the exe. Or edit the settings file. Open it via the
command palette with `zed: open settings file`. On Windows it lives at
`C:\Users\<username>\AppData\Roaming\Zed\settings.json`.

```json
{
  "context_servers": {
    "chhsich-web-search": {
      "command": "C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe",
      "args": []
    }
  }
}
```

Note that Zed's built-in `search_web` tool requires a Zed Pro subscription. On
a free plan this server provides the same capability.
</details>

<details>
<summary><strong>opencode:</strong> <code>mcp</code> key, <code>"type": "local"</code>, and <code>command</code> is an array</summary>

Config lives in `opencode.json` in the project root, or
`~/.config/opencode/opencode.json` globally. On Windows that is
`%USERPROFILE%\.config\opencode\opencode.json`.

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "chhsich-web-search": {
      "type": "local",
      "command": ["C:/Users/<username>/AppData/Local/Programs/agent-web-search/agent-web-search.exe"]
    }
  }
}
```
</details>

**Any other stdio MCP client.** Find where it keeps its MCP server list,
whether a JSON/YAML config or a settings UI, and add one entry: type `stdio`,
`command` = absolute path to the binary, `args` = `[]`. That's the whole
contract. There are no other parameters to set.

> **Replace the path.** The examples above use the recommended install location
> with `<username>` as a placeholder. Swap in your actual username, or use
> `%LOCALAPPDATA%` if your client expands environment variables. Adjust the
> path if you put the binary somewhere else.

> **Naming.** The key, `chhsich-web-search` in the examples, is your
> client-side label for the server, so call it whatever you want. The tool it
> exposes is always named `web_search`, so the model calls `web_search(...)`
> in every client.

> **Windows path tip.** Use the full absolute path including `.exe`. Forward
> slashes work in JSON and avoid backslash escaping.

> **Windows SmartScreen note.** The release binary is unsigned, so Windows may
> show a "Windows protected your PC" prompt the first time it runs. Click
> **More info → Run anyway**. This is expected for unsigned binaries and only
> happens once.

Restart your client after editing the config. The `web_search` tool now
appears alongside the built-in tools and the model can call it like any other.

### 3. Use it

First confirm the server actually registered. Every client has a status view:

- **Zed**: check Settings → AI → MCP Servers. The dot next to
  `chhsich-web-search` should be green, shown as "Server is active".
- **opencode**: run `opencode mcp list` in a terminal.
- **Claude Code**: run `claude mcp list`.

Then ask the model to search for anything, naming the server so it picks this
tool over any built-in one:

> Use the chhsich-web-search web_search tool to search for "rust async runtime"

Each result has a title, URL, site name, favicon, and a summary. For the top
hits the summary is a page-body extract.

**If the agent keeps using its own built-in web search.** Claude Code, Zed,
and opencode all ship a web-search tool, and registering this server does
*not* disable it. The model may keep picking the built-in one, which looks
exactly like the tool not working while the registration is fine. Two ways to
make it use this one:

- Name the server in the prompt as above, or add a line to your `AGENTS.md` or
  rules file: "When you need to search the web, use the `chhsich-web-search`
  `web_search` tool."
- Or turn off the built-in tool:
  - **Claude Code**: add `"WebSearch"` to `permissions.deny` in
    `~/.claude/settings.json`. A bare tool name removes it from the model's
    context entirely.
  - **Zed**: create an [agent
    profile](https://zed.dev/docs/ai/agent-profiles) where the built-in
    `search_web` is off and only this server's tools are on. On free plans
    `search_web` is unavailable anyway.
  - **opencode**: disable it via the `tools` key in `opencode.json`, for
    example `"tools": { "websearch": false }`.

If the server never shows up or calls fail, see [Troubleshooting](#troubleshooting).

## Usage

This is real output from a query for `rust tokio tutorial`, the top result
the query returned. The `summary` is a page-body extract, trimmed here for
display:

```json
{
  "title": "Tutorial | Tokio - An asynchronous Rust runtime",
  "url": "https://tokio.rs/tokio/tutorial",
  "summary": "Tokio is an asynchronous runtime for the Rust programming
    language. It provides the building blocks needed for writing networking
    applications. It gives the flexibility to target a wide range of systems,
    from large servers with dozens of cores to small embedde …",
  "site_name": "tokio.rs",
  "favicon": "https://tokio.rs/favicon.ico"
}
```

What you say in a prompt maps to parameters:

| You say … | Parameter |
| --- | --- |
| "only results from `docs.rust-lang.org`" | `search_domain_filter: "docs.rust-lang.org"` |
| "just from the last week" | `search_recency_filter: "oneWeek"` |
| "give me more of the page text" | `content_size: "high"` |
| "search English-language sources" | `location: "us"` |

Keyword queries beat questions. `rust tokio spawn example` returns sharply
more relevant results than "how do I spawn a task with tokio in rust?",
especially for Chinese queries. See [Known limitations](#known-limitations).

## Comparison

|  | agent-web-search | Hosted search APIs such as Tavily, Serper, Brave Search API |
| --- | --- | --- |
| API key | none | required |
| Cost | $0 per query, uncapped | metered, quota per plan |
| Where queries and page content go | your machine → DuckDuckGo and the sites directly | through the vendor's cloud |
| Runtime | one self-contained binary | account, key management, SDK updates |
| Published date, images, news in results | no, flat ranked web results | varies by service and plan |
| License | AGPL-3.0, dual-licensed | commercial SaaS terms |

Feature standing as of October 2026. Check each service's current pricing.

## Troubleshooting

- **The tool never appears.** Confirm the client config's `command` is the
  absolute path to the binary you downloaded, including the `.exe` on Windows.
  Then restart the client and check the status views in step 3.
- **Calls fail immediately.** The server returns every failure, whether a rate
  limit, empty results, or a page-fetch error, as a structured response the
  model can read, and the process never crashes. A transient upstream rate
  limit usually clears on its own, so retry after a short wait.
- **Results look empty or irrelevant.** Prefer keyword queries over
  natural-language questions, and add a domain filter when you know the
  source. See [Known limitations](#known-limitations).
- **Logs.** The server logs to stderr at INFO level and writes nothing to
  files. Stdout carries only JSON-RPC. Clients that display MCP server stderr
  show the logs there.

<details>
<summary><strong>FAQ: Free? What's the catch?</strong></summary>

No catch, just trade-offs. Search goes to DuckDuckGo over plain HTTP through
the `ddgs` library, with no key and no billing. In exchange you accept the
limitations above: best-effort recency filtering, no published-date field, and
upstream rate limits under heavy use.

</details>

<details>
<summary><strong>FAQ: Can I use this at work?</strong></summary>

Yes. Internal commercial use is free under AGPL-3.0. Distributing the tool
or offering it as a network service requires derivative works to be
open-sourced under AGPL-3.0, and closed-source commercial use needs a separate
commercial license. See [License](#license).

</details>

## How it works

```
query → map params → DuckDuckGo (ddgs: anti-bot/retry handled)
     → result list (title/url/snippet per hit)
     → top results: fetch page → Readability extract → word-limited body text
                                                        ↘ snippet fallback (never empty)
     → assemble {title, url, summary, site_name, favicon} → JSON
```

Search and page-fetch go through dependency-injected seams, so the core logic
for fan-out, extraction, and assembly is unit-tested without touching the
network. Every failure comes back as a structured response the model can read,
and the server process never crashes.

An earlier version of this project targeted SearXNG public instances in Rust,
but live testing found 0 of 38 instances usable, so it switched to DuckDuckGo
via `ddgs`.

## Contributing

Issues and PRs are welcome. Issues live as GitHub issues in this repo. See
`docs/agents/issue-tracker.md` for the conventions. A good bug report includes
the exact query, the parameters you passed, what you expected, and what came
back.

Local development:

```sh
pip install -e .
python -m pytest          # integration-marked tests hit the real network
```

If you're an AI agent or a new contributor, start with `CONTEXT.md` for the
glossary, `docs/adr/` for the architectural decisions, and `AGENTS.md` for the
repo conventions.

By submitting a PR, you agree to the [Contribution Terms](#contribution-terms).

## License

AGPL-3.0-only · Copyright (c) 2026 ChHsiching, see [LICENSE](LICENSE).

- Use, including internal commercial use, modification, and distribution is free. Distributing it or offering it as a network service requires derivative works to be open-sourced under AGPL-3.0.
- Closed-source commercial use requires a separate commercial license: hsichingchang@gmail.com

### Contribution Terms

By submitting a PR, you agree to license your contribution under AGPL-3.0 and grant the maintainer the right to offer separate commercial licenses. Your contribution remains available to everyone under AGPL.
