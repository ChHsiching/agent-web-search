## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

A GitHub Actions workflow that, on a version tag, builds a PyInstaller single-file executable per target platform and publishes them as GitHub Release assets: Windows, Linux, macOS. PyInstaller does not cross-compile, so each platform needs its own CI runner. Each asset is a standalone binary — users download one file and run it, with no Python install.

Also write/update the installation documentation: how to download the binary, place it on PATH, and configure it as an MCP server in Claude Code / Codex / ZCode. Target: set up in under a minute.

PyInstaller bundling of `ddgs` has been verified to work (real search returned results through a bundled exe, ~25MB, 1.3s startup).

## Acceptance criteria

- [ ] A GitHub Actions workflow triggers on version tag and builds via PyInstaller on Windows, Linux, and macOS runners.
- [ ] Each build produces a single-file executable published as a Release asset (compressed where idiomatic).
- [ ] `ddgs` and all dependencies are correctly bundled (hidden-imports handled) — verified the produced binary actually searches.
- [ ] Installation README documents: download, extract, PATH, and the MCP server config snippet for each target agent.
- [ ] A user following the README can go from download to a working `web_search_prime` tool call in under a minute.

## Blocked by

- #15 — Wire MCP tool to full pipeline + integration test (nothing is released before the tool works end-to-end).
