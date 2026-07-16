## Parent

#1 — Spec: agent-web-search — free MCP web-search tool for coding agents

## What to build

A GitHub Actions workflow that, on a version tag, cross-compiles the binary for the six target platforms and publishes them as GitHub Release assets: Windows (x64), Linux (x64, arm64), macOS (x64, arm64). Each is a pre-compiled static binary — the primary install path, so users download one file and run it, with no Rust toolchain, Docker, or Python required.

Also write the installation documentation: how to download the binary for your platform, place it on PATH, and configure it as an MCP server in Claude Code / Codex / ZCode (the one-line config change that makes it a drop-in replacement for `web_search_prime`). Target: set up in under a minute.

## Acceptance criteria

- [ ] A GitHub Actions workflow triggers on version tag push and cross-compiles for: `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`.
- [ ] Each build produces a standalone binary published as a Release asset (compressed where idiomatic, e.g. .zip for Windows, .tar.gz for Unix).
- [ ] Installation README documents: download the right asset, extract, place on PATH, and the MCP server config snippet for each of the three target agents.
- [ ] A user following the README can go from download to a working `web_search_prime` tool call in under a minute.
- [ ] `cargo install` via crates.io is documented as a secondary path (optional for this ticket; the pre-compiled binaries are the primary deliverable).

## Blocked by

- #8 — Wire MCP tool to full search pipeline (nothing is released before the tool works end-to-end).
